//! The network send and receive System

use std::{
    clone::Clone,
    io::{Error, ErrorKind},
    net::{SocketAddr, UdpSocket},
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

use serde::{de::DeserializeOwned, Serialize};

use amethyst_core::specs::{Join, Resources, System, SystemData, WriteStorage};

use super::{deserialize_event, send_event, ConnectionState, NetConnection, NetEvent, NetFilter};

enum InternalSocketEvent<E> {
    SendEvents {
        target: SocketAddr,
        events: Vec<NetEvent<E>>,
    },
    Stop,
}

struct RawEvent {
    pub byte_count: usize,
    pub data: Vec<u8>,
    pub source: SocketAddr,
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
    pub filters: Vec<Box<NetFilter<E>>>,

    tx: Sender<InternalSocketEvent<E>>,
    rx: Receiver<RawEvent>,
}

impl<E> NetSocketSystem<E>
where
    E: Serialize + PartialEq + Send + 'static,
{
    /// Creates a `NetSocketSystem` and binds the Socket on the ip and port added in parameters.
    pub fn new(addr: SocketAddr, filters: Vec<Box<NetFilter<E>>>) -> Result<Self, Error> {
        if addr.port() < 1024 {
            // Just warning the user here, just in case they want to use the root port.
            warn!("Using a port below 1024, this will require root permission and should not be done.");
        }

        let socket = UdpSocket::bind(addr)?;

        socket
            .set_nonblocking(true)
            .expect("Unable to set `UdpSocket` to non-blocking mode");

        // this -> thread
        let (tx1, rx1) = channel();
        // thread -> this
        let (tx2, rx2) = channel();

        thread::spawn(move || {
            //rx1,tx2
            let send_queue = rx1;
            let receive_queue = tx2;
            let socket = socket;

            'outer: loop {
                // send
                for control_event in send_queue.try_iter() {
                    match control_event {
                        InternalSocketEvent::SendEvents { target, events } => {
                            for ev in events {
                                send_event(&ev, &target, &socket);
                            }
                        }
                        InternalSocketEvent::Stop => break 'outer,
                    }
                }

                // receive
                let mut buf = [0 as u8; 2048];
                loop {
                    match socket.recv_from(&mut buf) {
                        // Data received
                        Ok((amt, src)) => {
                            if let Err(_) = receive_queue.send(RawEvent {
                                byte_count: amt,
                                data: buf[..amt].to_vec(),
                                source: src,
                            }) {
                                error!("`NetworkSocketSystem` was dropped");
                                break 'outer;
                            }
                        }
                        Err(e) => {
                            if e.kind() == ErrorKind::WouldBlock {
                                break;
                            } else {
                                error!("Could not receive datagram: {}", e);
                            }
                        }
                    }
                }
            }
        });

        Ok(NetSocketSystem {
            filters,
            tx: tx1,
            rx: rx2,
        })
    }
}

impl<'a, E> System<'a> for NetSocketSystem<E>
where
    E: Send + Sync + Serialize + Clone + DeserializeOwned + PartialEq + 'static,
{
    type SystemData = (WriteStorage<'a, NetConnection<E>>);

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
    }

    fn run(&mut self, mut net_connections: Self::SystemData) {
        for mut net_connection in (&mut net_connections).join() {
            let target = net_connection.target;

            if net_connection.state == ConnectionState::Connected
                || net_connection.state == ConnectionState::Connecting
            {
                self.tx
                    .send(InternalSocketEvent::SendEvents {
                        target,
                        events: net_connection.send_buffer_early_read().cloned().collect(),
                    }).expect("Unreachable: Channel will be alive until a stop event is sent");
            } else if net_connection.state == ConnectionState::Disconnected {
                self.tx
                    .send(InternalSocketEvent::Stop)
                    .expect("Already sent a stop event to the channel");
            }
        }

        for raw_event in self.rx.try_iter() {
            let mut matched = false;
            // Get the NetConnection from the source
            for mut net_connection in (&mut net_connections).join() {
                // We found the origin
                if net_connection.target == raw_event.source {
                    matched = true;
                    // Get the event
                    let net_event = deserialize_event::<E>(raw_event.data.as_slice());
                    match net_event {
                        Ok(ev) => {
                            net_connection.receive_buffer.single_write(ev);
                        }
                        Err(e) => error!(
                            "Failed to deserialize an incoming network event: {} From source: {:?}",
                            e, raw_event.source
                        ),
                    }
                }
                if !matched {
                    println!("Received packet from unknown source");
                }
            }
        }
    }
}
