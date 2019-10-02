//! Network systems implementation backed by the UDP network protocol.

use crate::simulation::{
    events::NetworkSimulationEvent,
    requirements::DeliveryRequirement,
    timing::{NetworkSimulationTime, NetworkSimulationTimeSystem},
    transport::{
        TransportResource, NETWORK_RECV_SYSTEM_NAME, NETWORK_SEND_SYSTEM_NAME,
        NETWORK_SIM_TIME_SYSTEM_NAME,
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
use std::{io, net::UdpSocket};

/// Use this network bundle to add the UDP transport layer to your game.
pub struct UdpNetworkBundle {
    socket: Option<UdpSocket>,
    recv_buffer_size_bytes: usize,
}

impl UdpNetworkBundle {
    pub fn new(socket: Option<UdpSocket>, recv_buffer_size_bytes: usize) -> Self {
        Self {
            socket,
            recv_buffer_size_bytes,
        }
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for UdpNetworkBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'_, '_>,
    ) -> Result<(), Error> {
        builder.add(UdpNetworkSendSystem, NETWORK_SEND_SYSTEM_NAME, &[]);
        builder.add(
            UdpNetworkRecvSystem::with_buffer_capacity(self.recv_buffer_size_bytes),
            NETWORK_RECV_SYSTEM_NAME,
            &[],
        );
        builder.add(
            NetworkSimulationTimeSystem,
            NETWORK_SIM_TIME_SYSTEM_NAME,
            &[NETWORK_SEND_SYSTEM_NAME, NETWORK_RECV_SYSTEM_NAME],
        );
        world.insert(UdpSocketResource::new(self.socket));
        Ok(())
    }
}

pub struct UdpNetworkSendSystem;

impl<'s> System<'s> for UdpNetworkSendSystem {
    type SystemData = (
        Write<'s, TransportResource>,
        Write<'s, UdpSocketResource>,
        Read<'s, NetworkSimulationTime>,
    );

    fn run(&mut self, (mut transport, mut socket, sim_time): Self::SystemData) {
        socket.get_mut().map(|socket| {
            let messages = transport.drain_messages_to_send(|_| sim_time.should_send_message_now());
            for message in messages.iter() {
                match message.delivery {
                    DeliveryRequirement::Unreliable | DeliveryRequirement::Default => {
                        if let Err(e) = socket.send_to(&message.payload, message.destination) {
                            error!("There was an error when attempting to send packet: {:?}", e);
                        }
                    }
                    delivery => panic!(
                        "{:?} is unsupported. UDP only supports Unreliable by design.",
                        delivery
                    ),
                }
            }
        });
    }
}

pub struct UdpNetworkRecvSystem {
    // TODO: Probably should move this to the UdpSocketResource
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
        Write<'s, UdpSocketResource>,
        Write<'s, EventChannel<NetworkSimulationEvent>>,
    );

    fn run(&mut self, (mut socket, mut event_channel): Self::SystemData) {
        socket.get_mut().map(|socket| {
            loop {
                match socket.recv_from(&mut self.recv_buffer) {
                    Ok((recv_len, address)) => {
                        let event = NetworkSimulationEvent::Message(
                            address,
                            Bytes::from(&self.recv_buffer[..recv_len]),
                        );
                        // TODO: Handle other types of events.
                        event_channel.single_write(event);
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

/// Resource to own the UDP socket.
pub struct UdpSocketResource {
    socket: Option<UdpSocket>,
}

impl Default for UdpSocketResource {
    fn default() -> Self {
        Self { socket: None }
    }
}

impl UdpSocketResource {
    /// Create a new instance of the `UdpSocketResource`
    pub fn new(socket: Option<UdpSocket>) -> Self {
        Self { socket }
    }

    /// Returns a mutable reference to the socket if there is one configured.
    pub fn get_mut(&mut self) -> Option<&mut UdpSocket> {
        self.socket.as_mut()
    }

    /// Sets the bound socket to the `UdpSocketResource`.
    pub fn set_socket(&mut self, socket: UdpSocket) {
        self.socket = Some(socket);
    }

    /// Drops the socket from the `UdpSocketResource`.
    pub fn drop_socket(&mut self) {
        self.socket = None;
    }
}
