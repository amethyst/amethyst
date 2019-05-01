//! The network send and receive System

use std::{clone::Clone, net::SocketAddr, thread};

use amethyst_core::ecs::{Join, Resources, System, SystemData, WriteStorage};

use crossbeam_channel::{Receiver, Sender};
use laminar::{Packet, SocketEvent};
use log::{error, warn};
use serde::{de::DeserializeOwned, Serialize};

use super::{
    deserialize_event,
    error::Result,
    send_event,
    server::{Host, ServerConfig},
    ConnectionState, NetConnection, NetEvent, NetFilter,
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
    /// The list of filters applied on the events received.
    pub filters: Vec<Box<dyn NetFilter<E>>>,
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
    pub fn new(config: ServerConfig, filters: Vec<Box<dyn NetFilter<E>>>) -> Result<Self> {
        if config.udp_socket_addr.port() < 1024 {
            // Just warning the user here, just in case they want to use the root port.
            warn!("Using a port below 1024, this will require root permission and should not be done.");
        }

        let server = Host::run(&config)?;

        let udp_send_handle = server.udp_send_handle();
        let udp_receive_handle = server.udp_receive_handle();

        let event_sender = NetSocketSystem::<E>::start_sending(udp_send_handle);

        Ok(NetSocketSystem {
            filters,
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
    type SystemData = (WriteStorage<'a, NetConnection<E>>);

    fn run(&mut self, mut net_connections: Self::SystemData) {
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
                    // Get the event
                    match deserialize_event::<E>(packet.payload()) {
                        Ok(event) => {
                            // Get the NetConnection from the source
                            for connection in (&mut net_connections).join() {
                                if connection.target_addr == packet.addr() {
                                    connection
                                        .receive_buffer
                                        .single_write(NetEvent::Packet(event.clone()));
                                };
                            }
                        }
                        Err(e) => error!(
                            "Failed to deserialize an incoming network event: {} From source: {:?}",
                            e,
                            packet.addr()
                        ),
                    };
                }
                SocketEvent::Connect(_) => { /* TODO: Update connection status */ }
                SocketEvent::Timeout(_) => { /* TODO: Update connection status */ }
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
