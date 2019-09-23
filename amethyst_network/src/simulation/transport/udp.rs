//! Network systems implementation backed by the UDP network protocol.
use crate::simulation::{
    events::NetworkSimulationEvent,
    requirements::DeliveryRequirement,
    resource::NetworkSimulationResource,
    timing::{NetworkSimulationTime, NetworkSimulationTimeSystem},
    transport::{
        run_network_recv_system, run_network_send_system, socket::Socket, NETWORK_RECV_SYSTEM_NAME,
        NETWORK_SEND_SYSTEM_NAME, NETWORK_SIM_TIME_SYSTEM_NAME,
    },
};
use amethyst_core::{
    bundle::SystemBundle,
    ecs::{DispatcherBuilder, Read, System, World, Write},
    shrev::EventChannel,
};
use amethyst_error::Error;
use bytes::Bytes;
use log::error;
use std::{
    io,
    net::UdpSocket,
    ops::{Deref, DerefMut},
};

/// Use this network bundle to add the various underlying UDP network systems to your game.
pub struct UdpNetworkBundle {
    recv_buffer_size_bytes: usize,
}

impl UdpNetworkBundle {
    pub fn new(recv_buffer_size_bytes: usize) -> Self {
        Self {
            recv_buffer_size_bytes,
        }
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for UdpNetworkBundle {
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
        builder.add(UdpNetworkSendSystem, NETWORK_SEND_SYSTEM_NAME, &[]);
        builder.add(
            UdpNetworkRecvSystem::with_buffer_capacity(self.recv_buffer_size_bytes),
            NETWORK_RECV_SYSTEM_NAME,
            &[],
        );
        Ok(())
    }
}

pub struct UdpNetworkSendSystem;

impl<'s> System<'s> for UdpNetworkSendSystem {
    type SystemData = (
        Write<'s, NetworkSimulationResource<UdpSocket>>,
        Read<'s, NetworkSimulationTime>,
    );

    fn run(&mut self, (mut net, sim_time): Self::SystemData) {
        run_network_send_system(
            net.deref_mut(),
            sim_time.deref(),
            |socket, addr, message| match message.delivery {
                DeliveryRequirement::Unreliable => {
                    if let Err(e) = socket.send_to(&message.payload, addr) {
                        error!("There was an error when attempting to send packet: {:?}", e);
                    }
                }
                delivery => panic!(
                    "{:?} is unsupported. UDP only supports Unreliable by design.",
                    delivery
                ),
            },
        );
    }
}

pub struct UdpNetworkRecvSystem {
    recv_buffer: Vec<u8>,
}

impl UdpNetworkRecvSystem {
    pub fn with_buffer_capacity(size: usize) -> Self {
        Self {
            recv_buffer: vec![0; size],
        }
    }
}

impl<'s> System<'s> for UdpNetworkRecvSystem {
    type SystemData = (
        Write<'s, NetworkSimulationResource<UdpSocket>>,
        Write<'s, EventChannel<NetworkSimulationEvent>>,
    );

    fn run(&mut self, (mut net, mut recv_channel): Self::SystemData) {
        run_network_recv_system(net.deref_mut(), |socket| {
            loop {
                match socket.recv_from(&mut self.recv_buffer) {
                    Ok((recv_len, address)) => {
                        let event = NetworkSimulationEvent::Message(
                            address,
                            Bytes::from(&self.recv_buffer[..recv_len]),
                        );
                        // TODO: Handle other types of events.
                        recv_channel.single_write(event);
                    }
                    Err(e) => {
                        if e.kind() != io::ErrorKind::WouldBlock {
                            error!("Encountered an error receiving data: {:?}", e);
                        }
                        break;
                    }
                }
            }
        });
    }
}

impl Socket for UdpSocket {
    fn default_requirement() -> DeliveryRequirement {
        DeliveryRequirement::Unreliable
    }
}
