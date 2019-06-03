//! The network send and receive System

use std::{clone::Clone, net::SocketAddr, thread};

use amethyst_core::ecs::{Entities, Join, Resources, System, SystemData, WriteStorage};

use crossbeam_channel::{Receiver, Sender};
use laminar::{Packet, SocketEvent};
use log::{error, warn};
use serde::{de::DeserializeOwned, Serialize};

use super::{
    error::Result,
    send_event,
    server::{Host, ServerConfig},
    ConnectionState, NetConnection, NetEvent,
};

enum InternalSocketEvent<E> {
    SendEvents {
        target: SocketAddr,
        events: Vec<NetEvent<E>>,
    },
    Stop,
}

// If a client sends both a connect event and other events,
// only the connect event will be considered valid and all others will be lost.
/// The System managing the network state and connections.
/// The T generic parameter corresponds to the network event type.
/// Receives events and filters them.
/// Received events will be inserted into the NetReceiveBuffer resource.
/// To send an event, add it to the NetSendBuffer resource.
///
/// If both a connection (Connect or Connected) event is received at the same time as another event from the same connection,
/// only the connection event will be considered and rest will be filtered out.
// TODO: add Unchecked Event type list. Those events will be let pass the client connected filter (Example: NetEvent::Connect).
// Current behaviour: hardcoded passthrough of Connect and Connected events.
pub struct NetSocketSystem<E: 'static>
where
    E: PartialEq,
{
    // sender on which you can queue packets to send to some endpoint.
    event_sender: Sender<InternalSocketEvent<E>>,
    // receiver from which you can read received packets.
    event_receiver: Receiver<laminar::SocketEvent>,
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
                            match ev {
                                NetEvent::Packet(packet) => {
                                    send_event(packet, target, &sender);
                                }
                                _ => { /* TODO, handle connect, disconnect etc. */ }
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

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
    }
}
