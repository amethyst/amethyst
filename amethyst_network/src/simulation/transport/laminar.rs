//! Network systems implementation backed by the Laminar network protocol.

use std::time::Instant;

use amethyst_core::{ecs::*, EventChannel};
use amethyst_error::Error;
use bytes::Bytes;
pub use laminar::{Config as LaminarConfig, ErrorKind, Socket as LaminarSocket};
use laminar::{Packet, SocketEvent};
use log::error;

use crate::simulation::{
    events::NetworkSimulationEvent,
    requirements::DeliveryRequirement,
    timing::{NetworkSimulationTime, NetworkSimulationTimeSystem},
    transport::TransportResource,
};

/// Use this network bundle to add the laminar transport layer to your game.
pub struct LaminarNetworkBundle {
    socket: Option<LaminarSocket>,
}

impl LaminarNetworkBundle {
    pub fn new(socket: Option<LaminarSocket>) -> Self {
        Self { socket }
    }
}

impl SystemBundle for LaminarNetworkBundle {
    fn load(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        resources.insert(LaminarSocketResource::new(self.socket.take()));

        builder
            .add_system(NetworkSimulationTimeSystem)
            .add_system(LaminarNetworkSendSystem)
            .add_system(LaminarNetworkPollSystem)
            .add_system(LaminarNetworkRecvSystem);

        Ok(())
    }
}

/// Creates a new laminar network send system.
pub struct LaminarNetworkSendSystem;

impl System for LaminarNetworkSendSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("LaminarNetworkSendSystem")
                .write_resource::<TransportResource>()
                .write_resource::<LaminarSocketResource>()
                .read_resource::<NetworkSimulationTime>()
                .write_resource::<EventChannel<NetworkSimulationEvent>>()
                .build(
                    move |_commands, _world, (transport, socket, sim_time, event_channel), _| {
                        if let Some(socket) = socket.get_mut() {
                            let messages = transport
                                .drain_messages_to_send(|_| sim_time.should_send_message_now());

                            for message in messages {
                                let packet = match message.delivery {
                                    DeliveryRequirement::Unreliable => {
                                        Packet::unreliable(
                                            message.destination,
                                            message.payload.to_vec(),
                                        )
                                    }
                                    DeliveryRequirement::UnreliableSequenced(stream_id) => {
                                        Packet::unreliable_sequenced(
                                            message.destination,
                                            message.payload.to_vec(),
                                            stream_id,
                                        )
                                    }
                                    DeliveryRequirement::Reliable => {
                                        Packet::reliable_unordered(
                                            message.destination,
                                            message.payload.to_vec(),
                                        )
                                    }
                                    DeliveryRequirement::ReliableSequenced(stream_id) => {
                                        Packet::reliable_sequenced(
                                            message.destination,
                                            message.payload.to_vec(),
                                            stream_id,
                                        )
                                    }
                                    DeliveryRequirement::ReliableOrdered(stream_id) => {
                                        Packet::reliable_ordered(
                                            message.destination,
                                            message.payload.to_vec(),
                                            stream_id,
                                        )
                                    }
                                    DeliveryRequirement::Default => {
                                        Packet::reliable_ordered(
                                            message.destination,
                                            message.payload.to_vec(),
                                            None,
                                        )
                                    }
                                };

                                match socket.send(packet) {
                                    Err(ErrorKind::IOError(e)) => {
                                        event_channel.single_write(
                                            NetworkSimulationEvent::SendError(e, message),
                                        );
                                    }
                                    Err(e) => {
                                        error!("Error sending message: {:?}", e);
                                    }
                                    Ok(_) => {}
                                }
                            }
                        }
                    },
                ),
        )
    }
}

/// Creates a new laminar network poll system.
pub struct LaminarNetworkPollSystem;

impl System for LaminarNetworkPollSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("LaminarNetworkPollSystem")
                .write_resource::<LaminarSocketResource>()
                .build(move |_commands, _world, socket, _| {
                    if let Some(socket) = socket.get_mut() {
                        socket.manual_poll(Instant::now());
                    }
                }),
        )
    }
}

/// Creates a new laminar receive system.
pub struct LaminarNetworkRecvSystem;

impl System for LaminarNetworkRecvSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("LaminarNetworkRecvSystem")
                .write_resource::<LaminarSocketResource>()
                .write_resource::<EventChannel<NetworkSimulationEvent>>()
                .build(move |_commands, _world, (socket, event_channel), _| {
                    if let Some(socket) = socket.get_mut() {
                        while let Some(event) = socket.recv() {
                            let event = match event {
                                SocketEvent::Packet(packet) => {
                                    NetworkSimulationEvent::Message(
                                        packet.addr(),
                                        Bytes::copy_from_slice(packet.payload()),
                                    )
                                }
                                SocketEvent::Disconnect(addr) => {
                                    NetworkSimulationEvent::Disconnect(addr)
                                }
                                SocketEvent::Connect(addr) => NetworkSimulationEvent::Connect(addr),
                                SocketEvent::Timeout(addr) => {
                                    NetworkSimulationEvent::Disconnect(addr)
                                }
                            };
                            event_channel.single_write(event);
                        }
                    }
                }),
        )
    }
}

/// Resource that owns the Laminar socket.
pub struct LaminarSocketResource {
    socket: Option<LaminarSocket>,
}

impl Default for LaminarSocketResource {
    fn default() -> Self {
        Self { socket: None }
    }
}

impl LaminarSocketResource {
    /// Creates a new instance of the `UdpSocketResource`.
    pub fn new(socket: Option<LaminarSocket>) -> Self {
        Self { socket }
    }

    /// Returns a reference to the socket if there is one configured.
    pub fn get(&self) -> Option<&LaminarSocket> {
        self.socket.as_ref()
    }

    /// Returns a mutable reference to the socket if there is one configured.
    pub fn get_mut(&mut self) -> Option<&mut LaminarSocket> {
        self.socket.as_mut()
    }

    /// Sets the bound socket to the `LaminarSocketResource`.
    pub fn set_socket(&mut self, socket: LaminarSocket) {
        self.socket = Some(socket);
    }

    /// Drops the socket from the `LaminarSocketResource`.
    pub fn drop_socket(&mut self) {
        self.socket = None;
    }
}
