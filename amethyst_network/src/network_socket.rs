//! The network send and receive System

use std::{clone::Clone, net::SocketAddr, thread};

use amethyst_core::ecs::{Entities, Join, System, WriteStorage};

use crossbeam_channel::{Receiver, Sender};
use laminar::{Packet, SocketEvent};
use log::{error, warn};
use serde::{de::DeserializeOwned, Serialize};

use super::{
    error::Result,
    serialize_event, serialize_packet,
    server::{Host, ServerConfig},
    ConnectionState, NetConnection, NetEvent,
};
use std::io::{Error, ErrorKind};

enum InternalSocketEvent<E> {
    SendEvents {
        target: SocketAddr,
        events: Vec<NetEvent<E>>,
    },
    Stop,
}

/// The System managing the network state from `NetConnections`.
///
/// This system has a few responsibilities.
///
/// - Reading to send packets from `NetConnection` and sending those over to some remote endpoint.
/// - Listening for incoming packets and queue the received packets (`NetEvent::Packet(...)`) on the accompanying `NetConnection`.
///
/// This system is able to create a `NetConnection` and add those to the world when a new client connects.
/// (This behavior might not be desired and can therefore be deactivated in the configuration).
///
/// In both cases when a client connects and disconnects a `NetEvent::Connected` or `NetEvent::Disconnected` will be queued on accompanying `NetConnection`
///
/// - `T` corresponds to the network event type.
#[allow(missing_debug_implementations)]
pub struct NetSocketSystem<E: 'static>
where
    E: PartialEq,
{
    // sender on which you can queue packets to send to some endpoint.
    event_sender: Sender<InternalSocketEvent<E>>,
    // receiver from which you can read received packets.
    event_receiver: Receiver<laminar::SocketEvent>,
    // the configuration with which you can configure the network behaviour.
    config: ServerConfig,
}

impl<E> NetSocketSystem<E>
where
    E: Serialize + PartialEq + Send + 'static,
{
    /// Creates a `NetSocketSystem` and binds the Socket on the ip and port added in parameters.
    pub fn new(config: ServerConfig) -> Result<Self> {
        if config.udp_socket_addr.port() < 1024 {
            // Just warning the user here, just in case they want to use the root port.
            warn!("Using a port below 1024, this will require root permission and should not be done.");
        }

        let server = Host::run(&config)?;

        let udp_send_handle = server.udp_send_handle();
        let udp_receive_handle = server.udp_receive_handle();

        let event_sender = NetSocketSystem::<E>::start_sending(udp_send_handle);

        Ok(NetSocketSystem {
            event_sender,
            event_receiver: udp_receive_handle,
            config,
        })
    }

    /// Start a thread to send all queued packets.
    fn start_sending(sender: Sender<Packet>) -> Sender<InternalSocketEvent<E>> {
        let (event_sender, event_receiver) = crossbeam_channel::unbounded();

        thread::spawn(move || loop {
            for control_event in event_receiver.try_iter() {
                match control_event {
                    InternalSocketEvent::SendEvents { target, events } => {
                        for ev in events {
                            let serialize_result = match ev {
                                NetEvent::Packet(packet) => serialize_packet(packet, target),
                                NetEvent::Connected(addr) => serialize_event(ev, addr),
                                NetEvent::Disconnected(addr) => serialize_event(ev, addr),
                                NetEvent::__Nonexhaustive => {
                                    Err(Error::new(ErrorKind::Other, "Net event does not exist.")
                                        .into())
                                }
                            };

                            match serialize_result {
                                Ok(packet) => match sender.send(packet) {
                                    Ok(_qty) => {}
                                    Err(e) => {
                                        error!("Failed to send data to network socket: {}", e)
                                    }
                                },
                                Err(e) => error!("Cannot serialize packet. Reason: {}", e),
                            }
                        }
                    }
                    InternalSocketEvent::Stop => {
                        break;
                    }
                }
            }
        });

        event_sender
    }
}

impl<'a, E> System<'a> for NetSocketSystem<E>
where
    E: Send + Sync + Serialize + Clone + DeserializeOwned + PartialEq + 'static,
{
    type SystemData = (WriteStorage<'a, NetConnection<E>>, Entities<'a>);

    fn run(&mut self, (mut net_connections, entities): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("net_socket_system");

        for connection in (&mut net_connections).join() {
            match connection.state {
                ConnectionState::Connected | ConnectionState::Connecting => {
                    self.event_sender
                        .send(InternalSocketEvent::SendEvents {
                            target: connection.target_addr,
                            events: connection.send_buffer_early_read().cloned().collect(),
                        })
                        .expect("Unreachable: Channel will be alive until a stop event is sent");
                }
                ConnectionState::Disconnected => {
                    self.event_sender
                        .send(InternalSocketEvent::Stop)
                        .expect("Already sent a stop event to the channel");
                }
            }
        }

        for (counter, socket_event) in self.event_receiver.try_iter().enumerate() {
            match socket_event {
                SocketEvent::Packet(packet) => {
                    let from_addr = packet.addr();

                    match NetEvent::<E>::from_packet(packet) {
                        Ok(event) => {
                            for connection in (&mut net_connections).join() {
                                if connection.target_addr == from_addr {
                                    connection.receive_buffer.single_write(event.clone());
                                }
                            }
                        }
                        Err(e) => error!(
                            "Failed to deserialize an incoming network event: {} From source: {:?}",
                            e, from_addr
                        ),
                    }
                }
                SocketEvent::Connect(addr) => {
                    if self.config.create_net_connection_on_connect {
                        let mut connection: NetConnection<E> = NetConnection::new(addr);
                        connection
                            .receive_buffer
                            .single_write(NetEvent::Connected(addr));

                        entities
                            .build_entity()
                            .with(connection, &mut net_connections)
                            .build();
                    }
                }
                SocketEvent::Timeout(timeout_addr) => {
                    for connection in (&mut net_connections).join() {
                        if connection.target_addr == timeout_addr {
                            // we can't remove the entity from the world here because it could still have events in it's buffer.
                            connection
                                .receive_buffer
                                .single_write(NetEvent::Disconnected(timeout_addr));
                        }
                    }
                }
            };

            // this will prevent our system to be stuck in the iterator.
            // After 10000 packets we will continue and leave the other packets for the next run.
            // eventually some congestion prevention should be done.
            if counter >= self.config.max_throughput as usize {
                break;
            }
        }
    }
}
