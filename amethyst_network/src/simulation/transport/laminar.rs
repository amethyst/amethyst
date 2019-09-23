//! Network systems implementation backed by the laminar network protocol.
use crate::simulation::{
    events::NetworkSimulationEvent,
    requirements::DeliveryRequirement,
    resource::NetworkSimulationResource,
    timing::{NetworkSimulationTime, NetworkSimulationTimeSystem},
    transport::{
        run_network_recv_system, run_network_send_system, socket::Socket, NETWORK_POLL_SYSTEM_NAME,
        NETWORK_RECV_SYSTEM_NAME, NETWORK_SEND_SYSTEM_NAME, NETWORK_SIM_TIME_SYSTEM_NAME,
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
use std::{
    ops::{Deref, DerefMut},
    time::Instant,
};

/// Use this network bundle to add the various underlying laminar network systems to your game.
pub struct LaminarNetworkBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for LaminarNetworkBundle {
    fn build(
        self,
        _world: &mut World,
        builder: &mut DispatcherBuilder<'_, '_>,
    ) -> Result<(), Error> {
        builder.add(
            NetworkSimulationTimeSystem,
            NETWORK_SIM_TIME_SYSTEM_NAME,
            &[],
        );
        builder.add(LaminarNetworkSendSystem, NETWORK_SEND_SYSTEM_NAME, &[]);
        builder.add(
            LaminarNetworkPollSystem,
            NETWORK_POLL_SYSTEM_NAME,
            &[NETWORK_SEND_SYSTEM_NAME],
        );
        builder.add(
            LaminarNetworkRecvSystem,
            NETWORK_RECV_SYSTEM_NAME,
            &[NETWORK_POLL_SYSTEM_NAME],
        );
        Ok(())
    }
}

struct LaminarNetworkSendSystem;

impl<'s> System<'s> for LaminarNetworkSendSystem {
    type SystemData = (
        Write<'s, NetworkSimulationResource<LaminarSocket>>,
        Read<'s, NetworkSimulationTime>,
    );

    fn run(&mut self, (mut net, sim_time): Self::SystemData) {
        run_network_send_system(
            net.deref_mut(),
            sim_time.deref(),
            |socket, addr, message| {
                let packet = match message.delivery {
                    DeliveryRequirement::Unreliable => {
                        Packet::unreliable(addr, message.payload.to_vec())
                    }
                    DeliveryRequirement::UnreliableSequenced(stream_id) => {
                        Packet::unreliable_sequenced(addr, message.payload.to_vec(), stream_id)
                    }
                    DeliveryRequirement::Reliable => {
                        Packet::reliable_unordered(addr, message.payload.to_vec())
                    }
                    DeliveryRequirement::ReliableSequenced(stream_id) => {
                        Packet::reliable_sequenced(addr, message.payload.to_vec(), stream_id)
                    }
                    DeliveryRequirement::ReliableOrdered(stream_id) => {
                        Packet::reliable_ordered(addr, message.payload.to_vec(), stream_id)
                    }
                };

                if let Err(e) = socket.send(packet) {
                    error!("There was an error when attempting to send packet: {:?}", e);
                }
            },
        );
    }
}

struct LaminarNetworkPollSystem;

impl<'s> System<'s> for LaminarNetworkPollSystem {
    type SystemData = Write<'s, NetworkSimulationResource<LaminarSocket>>;

    fn run(&mut self, mut net: Self::SystemData) {
        net.get_socket_mut()
            .map(|socket| socket.manual_poll(Instant::now()));
    }
}

struct LaminarNetworkRecvSystem;

impl<'s> System<'s> for LaminarNetworkRecvSystem {
    type SystemData = (
        Write<'s, NetworkSimulationResource<LaminarSocket>>,
        Write<'s, EventChannel<NetworkSimulationEvent>>,
    );

    fn run(&mut self, (mut net, mut event_channel): Self::SystemData) {
        run_network_recv_system(net.deref_mut(), |socket| {
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

impl Socket for LaminarSocket {
    fn default_requirement() -> DeliveryRequirement {
        DeliveryRequirement::ReliableOrdered(None)
    }
}
