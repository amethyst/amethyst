//! Network systems implementation backed by the TCP network protocol.

use std::{
    collections::HashMap,
    io::{self, Read as IORead, Write as IOWrite},
    net::{SocketAddr, TcpListener, TcpStream},
    ops::DerefMut,
};

use amethyst_core::{ecs::*, EventChannel};
use amethyst_error::Error;
use bytes::Bytes;
use log::warn;

use crate::simulation::{
    events::NetworkSimulationEvent,
    message::Message,
    requirements::DeliveryRequirement,
    timing::{NetworkSimulationTime, NetworkSimulationTimeSystem},
    transport::TransportResource,
};

/// Use this network bundle to add the TCP transport layer to your game.
pub struct TcpNetworkBundle {
    listener: Option<TcpListener>,
    recv_buffer_size_bytes: usize,
}

impl TcpNetworkBundle {
    pub fn new(listener: Option<TcpListener>, recv_buffer_size_bytes: usize) -> Self {
        Self {
            listener,
            recv_buffer_size_bytes,
        }
    }
}

impl SystemBundle for TcpNetworkBundle {
    fn load(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        resources.insert(TcpNetworkResource::new(
            self.listener.take(),
            self.recv_buffer_size_bytes,
        ));

        // NetworkSimulationTime should run first
        // followed by TcpConnectionListenerSystem and TcpStreamManagementSystem
        // then TcpNetworkSendSystem and TcpNetworkRecvSystem
        builder
            .add_system(NetworkSimulationTimeSystem)
            .add_system(TcpConnectionListenerSystem)
            .add_system(TcpStreamManagementSystem)
            .add_system(TcpNetworkSendSystem)
            .add_system(TcpNetworkRecvSystem);

        Ok(())
    }
}

/// Creates a new tcp stream management system
// We cannot use `net.streams.entry(message.destination).or_insert_with(|| { .. })` because
// there is a `return;` statement for early exit, which is not allowed within the closure.
pub struct TcpStreamManagementSystem;

#[allow(clippy::map_entry)]
impl System for TcpStreamManagementSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("TcpStreamManagementSystem")
                .write_resource::<TcpNetworkResource>()
                .read_resource::<TransportResource>()
                .write_resource::<EventChannel<NetworkSimulationEvent>>()
                .build(
                    move |_commands, _world, (net, transport, event_channel), _| {
                        // Make connections for each message in the channel if one hasn't yet been established
                        transport.get_messages().iter().for_each(|message| {
                            if !net.streams.contains_key(&message.destination) {
                                let s = match TcpStream::connect(message.destination) {
                                    Ok(s) => s,
                                    Err(e) => {
                                        event_channel.single_write(
                                            NetworkSimulationEvent::ConnectionError(
                                                e,
                                                Some(message.destination),
                                            ),
                                        );
                                        return;
                                    }
                                };
                                s.set_nonblocking(true).expect("Setting non-blocking mode");
                                s.set_nodelay(true).expect("Setting nodelay");
                                net.streams.insert(message.destination, (true, s));
                            }
                        });

                        // Remove inactive connections
                        net.streams.retain(|addr, (active, _)| {
                            if !*active {
                                event_channel
                                    .single_write(NetworkSimulationEvent::Disconnect(*addr));
                            }
                            *active
                        });
                    },
                ),
        )
    }
}

/// System to listen for incoming connections and cache them to the resource.
pub struct TcpConnectionListenerSystem;

impl System for TcpConnectionListenerSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("TcpConnectionListenerSystem")
                .write_resource::<TcpNetworkResource>()
                .write_resource::<EventChannel<NetworkSimulationEvent>>()
                .build(move |_commands, _world, (net, event_channel), _| {
                    let resource = net.deref_mut();
                    if let Some(ref listener) = resource.listener {
                        loop {
                            match listener.accept() {
                                Ok((stream, addr)) => {
                                    stream
                                        .set_nonblocking(true)
                                        .expect("Setting nonblocking mode");
                                    stream.set_nodelay(true).expect("Setting nodelay");
                                    resource.streams.insert(addr, (true, stream));
                                    event_channel
                                        .single_write(NetworkSimulationEvent::Connect(addr));
                                }
                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                    break;
                                }
                                Err(e) => {
                                    event_channel.single_write(
                                        NetworkSimulationEvent::ConnectionError(e, None),
                                    );
                                    break;
                                }
                            };
                        }
                    }
                }),
        )
    }
}

/// System to send messages to a particular open `TcpStream`.
pub struct TcpNetworkSendSystem;

impl System for TcpNetworkSendSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("TcpNetworkSendSystem")
                .write_resource::<TransportResource>()
                .write_resource::<TcpNetworkResource>()
                .read_resource::<NetworkSimulationTime>()
                .write_resource::<EventChannel<NetworkSimulationEvent>>()
                .build(
                    move |_commands, _world, (transport, net, sim_time, channel), _| {
                        let messages = transport
                            .drain_messages_to_send(|_| sim_time.should_send_message_now());
                        for message in messages {
                            match message.delivery {
                                DeliveryRequirement::ReliableOrdered(Some(_)) => {
                                    warn!("Streams are not supported by TCP and will be ignored.");
                                    write_message(message, net, channel);
                                }
                                DeliveryRequirement::ReliableOrdered(_)
                                | DeliveryRequirement::Default => {
                                    write_message(message, net, channel);
                                }
                                delivery => {
                                    panic!(
                            "{:?} is unsupported. TCP only supports ReliableOrdered by design.",
                            delivery
                        )
                                }
                            }
                        }
                    },
                ),
        )
    }
}

fn write_message(
    message: Message,
    net: &mut TcpNetworkResource,
    channel: &mut EventChannel<NetworkSimulationEvent>,
) {
    if let Some((_, stream)) = net.get_stream(message.destination) {
        if let Err(e) = stream.write(&message.payload) {
            channel.single_write(NetworkSimulationEvent::SendError(e, message));
        }
    }
}

/// System to receive messages from all open `TcpStream`s.
pub struct TcpNetworkRecvSystem;

impl System for TcpNetworkRecvSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("TcpNetworkRecvSystem")
                .write_resource::<TcpNetworkResource>()
                .write_resource::<EventChannel<NetworkSimulationEvent>>()
                .build(move |_commands, _world, (net, event_channel), _| {
                    let resource = net.deref_mut();
                    for (_, (active, stream)) in resource.streams.iter_mut() {
                        // If we can't get a peer_addr, there is likely something pretty wrong with the
                        // connection so we'll mark it inactive.
                        let peer_addr = match stream.peer_addr() {
                            Ok(addr) => addr,
                            Err(e) => {
                                warn!("Encountered an error getting peer_addr: {:?}", e);
                                *active = false;
                                continue;
                            }
                        };

                        loop {
                            match stream.read(&mut resource.recv_buffer) {
                                Ok(recv_len) => {
                                    if recv_len > 0 {
                                        let event = NetworkSimulationEvent::Message(
                                            peer_addr,
                                            Bytes::copy_from_slice(
                                                &resource.recv_buffer[..recv_len],
                                            ),
                                        );
                                        event_channel.single_write(event);
                                    } else {
                                        *active = false;
                                        break;
                                    }
                                }
                                Err(e) => {
                                    match e.kind() {
                                        io::ErrorKind::ConnectionReset => {
                                            *active = false;
                                        }
                                        io::ErrorKind::WouldBlock => {}
                                        _ => {
                                            event_channel
                                                .single_write(NetworkSimulationEvent::RecvError(e));
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }),
        )
    }
}

#[derive(Default)]
pub struct TcpNetworkResource {
    listener: Option<TcpListener>,
    streams: HashMap<SocketAddr, (bool, TcpStream)>,
    recv_buffer: Vec<u8>,
}

impl TcpNetworkResource {
    pub fn new(listener: Option<TcpListener>, recv_buffer_size_bytes: usize) -> Self {
        Self {
            listener,
            streams: HashMap::new(),
            recv_buffer: vec![0; recv_buffer_size_bytes],
        }
    }

    /// Returns an immutable reference to the listener if there is one configured.
    pub fn get(&self) -> Option<&TcpListener> {
        self.listener.as_ref()
    }

    /// Returns a mutable reference to the listener if there is one configured.
    pub fn get_mut(&mut self) -> Option<&mut TcpListener> {
        self.listener.as_mut()
    }

    /// Sets the bound listener to the `TcpNetworkResource`.
    pub fn set_listener(&mut self, listener: TcpListener) {
        self.listener = Some(listener);
    }

    /// Drops the listener from the `TcpNetworkResource`.
    pub fn drop_listener(&mut self) {
        self.listener = None;
    }

    /// Returns a tuple of an active TcpStream and whether ot not that stream is active
    pub fn get_stream(&mut self, addr: SocketAddr) -> Option<&mut (bool, TcpStream)> {
        self.streams.get_mut(&addr)
    }

    /// Drops the stream with the given `SocketAddr`. This will be called when a peer seems to have
    /// been disconnected
    pub fn drop_stream(&mut self, addr: SocketAddr) -> Option<(bool, TcpStream)> {
        self.streams.remove(&addr)
    }
}
