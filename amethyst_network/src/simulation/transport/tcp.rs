//! Network systems implementation backed by the TCP network protocol.

use crate::simulation::{
    events::NetworkSimulationEvent,
    requirements::DeliveryRequirement,
    timing::{NetworkSimulationTime, NetworkSimulationTimeSystem},
    transport::{
        TransportResource, NETWORK_RECV_SYSTEM_NAME, NETWORK_SEND_SYSTEM_NAME,
        NETWORK_SIM_TIME_SYSTEM_NAME,
    },
    Message,
};
use amethyst_core::{
    bundle::SystemBundle,
    ecs::{DispatcherBuilder, Read, System, World, Write},
    shrev::EventChannel,
};
use amethyst_error::Error;
use bytes::Bytes;
use log::{error, warn};
use std::{
    collections::HashMap,
    io::{self, Read as IORead, Write as IOWrite},
    net::{SocketAddr, TcpListener, TcpStream},
    ops::DerefMut,
};

const CONNECTION_LISTENER_SYSTEM_NAME: &str = "connection_listener";
const STREAM_MANAGEMENT_SYSTEM_NAME: &str = "stream_management";

/// Use this network bundle to add the various underlying tcp network systems to your game.
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

impl<'a, 'b> SystemBundle<'a, 'b> for TcpNetworkBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'_, '_>,
    ) -> Result<(), Error> {
        builder.add(TcpNetworkSendSystem, NETWORK_SEND_SYSTEM_NAME, &[]);
        builder.add(TcpNetworkRecvSystem, NETWORK_RECV_SYSTEM_NAME, &[]);
        builder.add(
            TcpStreamManagementSystem,
            STREAM_MANAGEMENT_SYSTEM_NAME,
            &[NETWORK_SEND_SYSTEM_NAME, NETWORK_RECV_SYSTEM_NAME],
        );
        builder.add(
            TcpConnectionListenerSystem,
            CONNECTION_LISTENER_SYSTEM_NAME,
            &[NETWORK_SEND_SYSTEM_NAME, NETWORK_RECV_SYSTEM_NAME],
        );
        builder.add(
            NetworkSimulationTimeSystem,
            NETWORK_SIM_TIME_SYSTEM_NAME,
            &[
                STREAM_MANAGEMENT_SYSTEM_NAME,
                CONNECTION_LISTENER_SYSTEM_NAME,
            ],
        );
        world.insert(TcpNetworkResource::new(
            self.listener,
            self.recv_buffer_size_bytes,
        ));
        Ok(())
    }
}

/// System to manage the current active TCPStreams.
pub struct TcpStreamManagementSystem;

impl<'s> System<'s> for TcpStreamManagementSystem {
    type SystemData = (
        Write<'s, TcpNetworkResource>,
        Read<'s, TransportResource>,
        Write<'s, EventChannel<NetworkSimulationEvent>>,
    );

    fn run(&mut self, (mut net, transport, mut event_channel): Self::SystemData) {
        // Make connections for each message in the channel if one hasn't yet been established
        transport.get_messages().iter().for_each(|message| {
            if !net.streams.contains_key(&message.destination) {
                let s = match TcpStream::connect(message.destination) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!(
                            "Error attempting to connection to {}: {:?}",
                            message.destination, e
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
                event_channel.single_write(NetworkSimulationEvent::Disconnect(*addr));
            }
            *active
        });
    }
}

/// System to listen for incoming connections and cache them to the resource.
pub struct TcpConnectionListenerSystem;

impl<'s> System<'s> for TcpConnectionListenerSystem {
    type SystemData = (
        Write<'s, TcpNetworkResource>,
        Write<'s, EventChannel<NetworkSimulationEvent>>,
    );

    fn run(&mut self, (mut net, mut event_channel): Self::SystemData) {
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
                        event_channel.single_write(NetworkSimulationEvent::Connect(addr));
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        break;
                    }
                    Err(e) => {
                        error!("Error listening for connections: {}", e);
                        break;
                    }
                };
            }
        }
    }
}

/// System to send messages to a particular open `TcpStream`.
pub struct TcpNetworkSendSystem;

impl<'s> System<'s> for TcpNetworkSendSystem {
    type SystemData = (
        Write<'s, TransportResource>,
        Write<'s, TcpNetworkResource>,
        Read<'s, NetworkSimulationTime>,
    );

    fn run(&mut self, (mut transport, mut net, sim_time): Self::SystemData) {
        let messages = transport.drain_messages_to_send(|_| sim_time.should_send_message_now());
        for message in messages.iter() {
            match message.delivery {
                DeliveryRequirement::ReliableOrdered(Some(_)) => {
                    warn!("Streams are not supported by TCP and will be ignored.");
                    write_message(message, &mut net);
                }
                DeliveryRequirement::ReliableOrdered(_) | DeliveryRequirement::Default => {
                    write_message(message, &mut net);
                }
                delivery => panic!(
                    "{:?} is unsupported. TCP only supports ReliableOrdered by design.",
                    delivery
                ),
            }
        }
    }
}

fn write_message(message: &Message, net: &mut TcpNetworkResource) {
    net.get_stream(message.destination).map(|(_, stream)| {
        if let Err(e) = stream.write(&message.payload) {
            error!(
                "There was an error when attempting to send message: {:?}",
                e
            );
        }
    });
}

/// System to receive messages from all open `TcpStream`s.
pub struct TcpNetworkRecvSystem;

impl<'s> System<'s> for TcpNetworkRecvSystem {
    type SystemData = (
        Write<'s, TcpNetworkResource>,
        Write<'s, EventChannel<NetworkSimulationEvent>>,
    );

    fn run(&mut self, (mut net, mut event_channel): Self::SystemData) {
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
                                Bytes::from(&resource.recv_buffer[..recv_len]),
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
                            _ => error!("Encountered an error receiving data: {:?}", e),
                        }
                        break;
                    }
                }
            }
        }
    }
}

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

    /// Return a mutable reference to the listener if there is one configured.
    pub fn get_mut(&mut self) -> Option<&mut TcpListener> {
        self.listener.as_mut()
    }

    /// Set the bound listener to the `TcpNetworkResource`
    pub fn set_listener(&mut self, listener: TcpListener) {
        self.listener = Some(listener);
    }

    /// Drops the listener from the `TcpNetworkResource`
    pub fn drop_listener(&mut self) {
        self.listener = None;
    }

    /// Returns a tuple of an active TcpStream and whether ot not that stream is active
    pub fn get_stream(&mut self, addr: SocketAddr) -> Option<&mut (bool, TcpStream)> {
        self.streams.get_mut(&addr)
    }

    /// Drop the stream with the given `SocketAddr`. This will be called when a peer seems to have
    /// been disconnected
    pub fn drop_stream(&mut self, addr: SocketAddr) -> Option<(bool, TcpStream)> {
        self.streams.remove(&addr)
    }
}

impl Default for TcpNetworkResource {
    fn default() -> Self {
        Self {
            listener: None,
            streams: HashMap::new(),
            recv_buffer: Vec::new(),
        }
    }
}
