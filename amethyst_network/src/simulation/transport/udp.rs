//! Network systems implementation backed by the UDP network protocol.

use crate::simulation::{
    events::NetworkSimulationEvent,
    requirements::DeliveryRequirement,
    timing::{NetworkSimulationTime, build_network_simulation_time_system},
    transport::{
        TransportResource, NETWORK_RECV_SYSTEM_NAME, NETWORK_SEND_SYSTEM_NAME,
        NETWORK_SIM_TIME_SYSTEM_NAME,
    },
};
use amethyst_core::{
    dispatcher::*,
    ecs::prelude::*,
    shrev::EventChannel,
};
use amethyst_error::Error;
use bytes::Bytes;
use std::{io, net::UdpSocket};

/// Use this network bundle to add the UDP transport layer to your game.
#[derive(new)]
pub struct UdpNetworkBundle {
    socket: Option<UdpSocket>,
    recv_buffer_size_bytes: usize,
}

impl SystemBundle for UdpNetworkBundle {
    fn build(
        self,
        world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder<'_>,
    ) -> Result<(), Error> {
        builder.add_system(Stage::Begin, build_network_simulation_time_system);
        builder.add_system(Stage::Begin, build_udp_network_receive_system);
        builder.add_system(Stage::Begin, build_udp_network_send_system);

        world.insert_resource(UdpSocketResource::new(self.socket, self.recv_buffer_size_bytes));
        Ok(())
    }
}

/// Creates a new network simulation time system.
pub fn build_udp_network_send_system(_world: &mut World, _res: &mut Resources) -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("UdpNetworkSendSystem")
        .write_resource::<TransportResource>()
        .write_resource::<UdpSocketResource>()
        .read_resource::<NetworkSimulationTime>()
        .write_resource::<EventChannel<NetworkSimulationEvent>>()
        .build(
            move |_commands,
                  world,
                  (transport, socket, sim_time, channel),
                  _| {
            if let Some(socket) = socket.get_mut() {
                let messages = transport.drain_messages_to_send(|_| sim_time.should_send_message_now());
                for message in messages {
                    match message.delivery {
                        DeliveryRequirement::Unreliable | DeliveryRequirement::Default => {
                            if let Err(e) = socket.send_to(&message.payload, message.destination) {
                                channel.single_write(NetworkSimulationEvent::SendError(e, message));
                            }
                        }
                        delivery => panic!(
                            "{:?} is unsupported. UDP only supports Unreliable by design.",
                            delivery
                        ),
                    }
                }
            }
    })
}


/// Creates a new udp network receiver system
pub fn build_udp_network_receive_system(_world: &mut World, _res: &mut Resources) -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("AudioSystem")
        .write_resource::<UdpSocketResource>()
        .write_resource::<EventChannel<NetworkSimulationEvent>>()
        .build(
            move |_commands,
                  world,
                  (socket, event_channel),
                  _| {
            if let Some(socket) = socket.get_mut() {
                loop {
                    match socket.recv_from(&mut socket.recv_buffer) {
                        Ok((recv_len, address)) => {
                            let event = NetworkSimulationEvent::Message(
                                address,
                                Bytes::copy_from_slice(&socket.recv_buffer[..recv_len]),
                            );
                            // TODO: Handle other types of events.
                            event_channel.single_write(event);
                        }
                        Err(e) => {
                            if e.kind() != io::ErrorKind::WouldBlock {
                                event_channel.single_write(NetworkSimulationEvent::RecvError(e));
                            }
                            break;
                        }
                    }
                }
            }
    })
}

/// Resource to own the UDP socket.
pub struct UdpSocketResource {
    socket: Option<UdpSocket>,
    recv_buffer: Vec<u8>,
}

impl Default for UdpSocketResource {
    fn default() -> Self {
        Self { socket: None, ..Default::default() }
    }
}

impl UdpSocketResource {
    /// Create a new instance of the `UdpSocketResource`
    pub fn new(socket: Option<UdpSocket>, size: usize) -> Self {
        Self { socket, recv_buffer: Vec::with_capacity(size) }
    }

    /// Returns an immutable reference to the socket if there is one configured.
    pub fn get(&self) -> Option<&UdpSocket> {
        self.socket.as_ref()
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

