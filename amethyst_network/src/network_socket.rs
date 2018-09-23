//! The network send and receive System

use super::{deserialize_event, send_event, ConnectionState, NetConnection, NetEvent};
use amethyst_core::specs::{Entities, Entity, Join, Resources, System, SystemData, WriteStorage};
use serde::de::DeserializeOwned;
use serde::Serialize;
use shrev::*;
use std::clone::Clone;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::str;
use std::str::FromStr;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

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
    /// The network socket, currently supports Udp only for demonstration purposes.
    // TODO: figure out why this is uncommented
    //pub socket: UdpSocket,

    /// The list of filters applied on the events received.
    //pub filters: Vec<Box<NetFilter<T>>>,

    /// The mio's `Poll`.
    //pub poll: Poll,
    tx: Sender<InternalSocketEvent<E>>,
    rx: Receiver<RawEvent>,

    /// Readers corresponding to each of the Connections. Use to keep track of when to send which event to who.
    /// When: When there is a new event that hasn't been read yet.
    /// Which: The event
    /// Who: The NetConnection's SocketAddr attached to the key Entity.
    send_queues_readers: HashMap<Entity, ReaderId<NetEvent<E>>>,
}

impl<E> NetSocketSystem<E>
where
    E: Serialize + PartialEq + Send + 'static,
{
    /// Creates a `NetSocketSystem` and binds the Socket on the ip and port added in parameters.
    pub fn new(
        ip: &str,
        port: u16,
        //filters: Vec<Box<NetFilter<T>>>,
    ) -> Result<NetSocketSystem<E>, Error> {
        let socket = UdpSocket::bind(&SocketAddr::new(
            IpAddr::from_str(ip).expect("Unreadable input IP."),
            port,
        ))?;
        socket.set_nonblocking(true).unwrap();

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
                                //info!("Sending event");
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
                            receive_queue
                                .send(RawEvent {
                                    byte_count: amt,
                                    data: buf[..amt].iter().cloned().collect(),
                                    source: src,
                                }).unwrap();
                        }
                        Err(e) => {
                            if e.kind() == ErrorKind::WouldBlock {
                                //error!("WouldBlock: {}", e);
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
            //socket,
            //filters,
            //poll,
            tx: tx1,
            rx: rx2,
            send_queues_readers: HashMap::new(),
        })
    }
}

impl<'a, E> System<'a> for NetSocketSystem<E>
where
    E: Send + Sync + Serialize + Clone + DeserializeOwned + PartialEq + 'static,
{
    type SystemData = (Entities<'a>, WriteStorage<'a, NetConnection<E>>);

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
    }

    fn run(&mut self, (entities, mut net_connections): Self::SystemData) {
        //self.socket.set_nonblocking(false).unwrap();
        for (entity, mut net_connection) in (&*entities, &mut net_connections).join() {
            // TODO: find out why the read needs this

            let mut _reader = self
                .send_queues_readers
                .entry(entity)
                .or_insert(net_connection.send_buffer.register_reader());
            let target = net_connection.target.clone();
            if net_connection.state == ConnectionState::Connected
                || net_connection.state == ConnectionState::Connecting
            {
                self.tx
                    .send(InternalSocketEvent::SendEvents {
                        target,
                        events: net_connection.send_buffer_early_read().cloned().collect(),
                    }).unwrap();
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
                            // Filter events
                            let mut filtered = false;

                            if !filtered {
                                net_connection.receive_buffer.single_write(ev);
                            } else {
                                info!(
                                    "Filtered an incoming network packet from source {:?}",
                                    raw_event.source
                                );
                            }
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
