//! Network systems implementation backed by the Laminar network protocol.

use crate::simulation::{
    events::NetworkSimulationEvent,
    requirements::DeliveryRequirement,
    timing::{NetworkSimulationTime, NetworkSimulationTimeSystem},
    transport::{
        TransportResource, NETWORK_POLL_SYSTEM_NAME, NETWORK_RECV_SYSTEM_NAME,
        NETWORK_SEND_SYSTEM_NAME, NETWORK_SIM_TIME_SYSTEM_NAME,
    },
};
use amethyst_core::{
    bundle::SystemBundle,
    ecs::{DispatcherBuilder, Read, System, World, Write},
    shrev::EventChannel,
};
use amethyst_error::Error;
pub use laminar::{Config as LaminarConfig, Socket as LaminarSocket};
use laminar::{Packet, SocketEvent};

use bytes::Bytes;
use log::error;
use std::time::Instant;

/// Use this network bundle to add the various underlying laminar network systems to your game.
pub struct LaminarNetworkBundle {
    socket: Option<LaminarSocket>,
}

impl LaminarNetworkBundle {
    pub fn new(socket: Option<LaminarSocket>) -> Self {
        Self { socket }
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for LaminarNetworkBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'_, '_>,
    ) -> Result<(), Error> {
        builder.add(
            NetworkSimulationTimeSystem,
            NETWORK_SIM_TIME_SYSTEM_NAME,
            &[],
        );
        builder.add(
            LaminarNetworkSendSystem,
            NETWORK_SEND_SYSTEM_NAME,
            &[NETWORK_SIM_TIME_SYSTEM_NAME],
        );
        builder.add(
            LaminarNetworkPollSystem,
            NETWORK_POLL_SYSTEM_NAME,
            &[NETWORK_SIM_TIME_SYSTEM_NAME, NETWORK_SEND_SYSTEM_NAME],
        );
        builder.add(
            LaminarNetworkRecvSystem,
            NETWORK_RECV_SYSTEM_NAME,
            &[NETWORK_SIM_TIME_SYSTEM_NAME, NETWORK_POLL_SYSTEM_NAME],
        );
        world.insert(LaminarSocketResource::new(self.socket));
        Ok(())
    }
}

struct LaminarNetworkSendSystem;

impl<'s> System<'s> for LaminarNetworkSendSystem {
    type SystemData = (
        Write<'s, TransportResource>,
        Write<'s, LaminarSocketResource>,
        Read<'s, NetworkSimulationTime>,
    );

    fn run(&mut self, (mut transport, mut socket, sim_time): Self::SystemData) {
        socket.get_mut().map(|socket| {
            let messages = transport.messages_to_send(|_| sim_time.should_send_messages());

            for message in messages.iter() {
                let packet = match message.delivery {
                    DeliveryRequirement::Unreliable => {
                        Packet::unreliable(message.destination, message.payload.to_vec())
                    }
                    DeliveryRequirement::UnreliableSequenced(stream_id) => {
                        Packet::unreliable_sequenced(
                            message.destination,
                            message.payload.to_vec(),
                            stream_id,
                        )
                    }
                    DeliveryRequirement::Reliable => {
                        Packet::reliable_unordered(message.destination, message.payload.to_vec())
                    }
                    DeliveryRequirement::ReliableSequenced(stream_id) => {
                        Packet::reliable_sequenced(
                            message.destination,
                            message.payload.to_vec(),
                            stream_id,
                        )
                    }
                    DeliveryRequirement::ReliableOrdered(stream_id) => Packet::reliable_ordered(
                        message.destination,
                        message.payload.to_vec(),
                        stream_id,
                    ),
                    DeliveryRequirement::Default => Packet::reliable_ordered(
                        message.destination,
                        message.payload.to_vec(),
                        None,
                    ),
                };

                if let Err(e) = socket.send(packet) {
                    error!("There was an error when attempting to send packet: {:?}", e);
                }
            }
        });
    }
}

struct LaminarNetworkPollSystem;

impl<'s> System<'s> for LaminarNetworkPollSystem {
    type SystemData = Write<'s, LaminarSocketResource>;

    fn run(&mut self, mut socket: Self::SystemData) {
        socket
            .get_mut()
            .map(|socket| socket.manual_poll(Instant::now()));
    }
}

struct LaminarNetworkRecvSystem;

impl<'s> System<'s> for LaminarNetworkRecvSystem {
    type SystemData = (
        Write<'s, LaminarSocketResource>,
        Write<'s, EventChannel<NetworkSimulationEvent>>,
    );

    fn run(&mut self, (mut socket, mut event_channel): Self::SystemData) {
        socket.get_mut().map(|socket| {
            while let Some(event) = socket.recv() {
                let event = match event {
                    SocketEvent::Packet(packet) => NetworkSimulationEvent::Message(
                        packet.addr(),
                        Bytes::from(packet.payload()),
                    ),
                    SocketEvent::Connect(addr) => NetworkSimulationEvent::Connect(addr),
                    SocketEvent::Timeout(addr) => NetworkSimulationEvent::Disconnect(addr),
                };
                event_channel.single_write(event);
            }
        });
    }
}

/// Resource to own the Laminar socket.
pub struct LaminarSocketResource {
    socket: Option<LaminarSocket>,
}

impl Default for LaminarSocketResource {
    fn default() -> Self {
        Self { socket: None }
    }
}

impl LaminarSocketResource {
    /// Create a new instance of the `UdpSocketResource`
    pub fn new(socket: Option<LaminarSocket>) -> Self {
        Self { socket }
    }

    /// Return a mutable reference to the socket if there is one configured.
    pub fn get_mut(&mut self) -> Option<&mut LaminarSocket> {
        self.socket.as_mut()
    }

    /// Set the bound socket to the `LaminarSocketResource`
    pub fn set_socket(&mut self, socket: LaminarSocket) {
        self.socket = Some(socket);
    }

    /// Drops the socket from the `LaminarSocketResource`
    pub fn drop_socket(&mut self) {
        self.socket = None;
    }
}
