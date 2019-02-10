//! The network send and receive System

use std::{
    clone::Clone,
    net::SocketAddr,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use amethyst_core::specs::{Join, Resources, System, SystemData, WriteStorage};

use laminar::Packet;
use log::{error, warn};
use serde::{de::DeserializeOwned, Serialize};

use super::{
    deserialize_event,
    error::Result,
    send_event,
    server::{Host, ReceiveHandler, SendHandler, ServerConfig, ServerSocketEvent},
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
    transport_sender: Sender<InternalSocketEvent<E>>,
    // receiver from which you can read received packets.
    transport_receiver: Receiver<Packet>,
    config: ServerConfig,
}

impl<E> NetSocketSystem<E>
where
    E: Serialize + PartialEq + Send + 'static,
{
    /// Creates a `NetSocketSystem` and binds the Socket on the ip and port added in parameters.
    pub fn new(config: ServerConfig, filters: Vec<Box<dyn NetFilter<E>>>) -> Result<Self> {
        if config.udp_recv_addr.port() < 1024 {
            // Just warning the user here, just in case they want to use the root port.
            warn!("Using a port below 1024, this will require root permission and should not be done.");
        }

        let server = Host::run(&config)?;

        let udp_send_handle = server.udp_send_handle();
        let udp_receive_handle = server.udp_receive_handle();

        let server_sender = NetSocketSystem::<E>::start_sending(udp_send_handle);
        let server_receiver = NetSocketSystem::<E>::start_receiving(udp_receive_handle);

        Ok(NetSocketSystem {
            filters,
            transport_sender: server_sender,
            transport_receiver: server_receiver,
            config,
        })
    }

    /// Start a thread to send all queued packets.
    fn start_sending(sender: Arc<SendHandler>) -> Sender<InternalSocketEvent<E>> {
        let (tx, send_queue) = mpsc::channel();

        thread::spawn(move || loop {
            for control_event in send_queue.try_iter() {
                match control_event {
                    InternalSocketEvent::SendEvents { target, events } => {
                        for ev in events {
                            send_event(ev, target, &sender.get_sender());
                        }
                    }
                    InternalSocketEvent::Stop => {
                        break;
                    }
                }
            }
        });

        tx
    }

    /// Starts a thread which receives incoming packets and sends them onto the 'Receiver' channel.
    fn start_receiving(receiver: Arc<Mutex<ReceiveHandler>>) -> Receiver<Packet> {
        let (receive_queue, rx) = mpsc::channel();

        thread::spawn(move || {
            // take note that this lock will be there as long this thread lives.
            // We only have one receiver, if one tries to have two receivers the program should panic.
            // Why is there one receiver? This is because how a channel works. Once we read the messages it will be removed from the channel.
            match receiver.try_lock() {
                Ok(receiver) => loop {
                    for event in receiver.iter() {
                        match event {
                            ServerSocketEvent::Packet(packet) => {
                                if let Err(error) = receive_queue.send(packet.clone()) {
                                    error!(
                                        "`NetworkSocketSystem` was dropped. Reason: {:?}",
                                        error
                                    );
                                    break;
                                }
                            }
                            ServerSocketEvent::Error(error) => error!("{:?}", error),
                            _ => error!("Event not supported"),
                        }
                    }
                },
                Err(_) => {
                    panic!("Two packet receivers can't run at the same time");
                }
            }
        });

        rx
    }
}

impl<'a, E> System<'a> for NetSocketSystem<E>
where
    E: Send + Sync + Serialize + Clone + DeserializeOwned + PartialEq + 'static,
{
    type SystemData = (WriteStorage<'a, NetConnection<E>>);

    fn run(&mut self, mut net_connections: Self::SystemData) {
        for net_connection in (&mut net_connections).join() {
            let target = net_connection.target_receiver;

            if net_connection.state == ConnectionState::Connected
                || net_connection.state == ConnectionState::Connecting
            {
                self.transport_sender
                    .send(InternalSocketEvent::SendEvents {
                        target,
                        events: net_connection.send_buffer_early_read().cloned().collect(),
                    })
                    .expect("Unreachable: Channel will be alive until a stop event is sent");
            } else if net_connection.state == ConnectionState::Disconnected {
                self.transport_sender
                    .send(InternalSocketEvent::Stop)
                    .expect("Already sent a stop event to the channel");
            }
        }

        for (counter, raw_event) in self.transport_receiver.try_iter().enumerate() {
            // Get the NetConnection from the source
            for net_connection in (&mut net_connections).join() {
                if net_connection.target_sender == raw_event.addr() {
                    // Get the event
                    match deserialize_event::<E>(raw_event.payload()) {
                        Ok(ev) => {
                            net_connection.receive_buffer.single_write(ev);
                        }
                        Err(e) => error!(
                            "Failed to deserialize an incoming network event: {} From source: {:?}",
                            e,
                            raw_event.addr()
                        ),
                    }
                } else {
                    warn!("Received packet from unknown source");
                }
            }

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
