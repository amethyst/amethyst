//! Network systems implementation backed by the UDP network protocol.

use std::{io, net::UdpSocket};

use amethyst_core::{ecs::*, EventChannel};
use amethyst_error::Error;
use bytes::Bytes;

use crate::simulation::{
    events::NetworkSimulationEvent,
    requirements::DeliveryRequirement,
    timing::{NetworkSimulationTime, NetworkSimulationTimeSystem},
    transport::TransportResource,
};

/// Use this network bundle to add the UDP transport layer to your game.
#[derive(new)]
pub struct UdpNetworkBundle {
    socket: Option<UdpSocket>,
    recv_buffer_size_bytes: usize,
}

impl SystemBundle for UdpNetworkBundle {
    fn load(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        resources.insert(UdpSocketResource::new(
            self.socket.take(),
            self.recv_buffer_size_bytes,
        ));

        builder
            .add_system(NetworkSimulationTimeSystem)
            .add_system(UdpNetworkReceiveSystem)
            .add_system(UdpNetworkSendSystem);

        Ok(())
    }
}

/// Creates a new network simulation time system.
pub struct UdpNetworkSendSystem;

impl System for UdpNetworkSendSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("UdpNetworkSendSystem")
                .write_resource::<TransportResource>()
                .write_resource::<UdpSocketResource>()
                .read_resource::<NetworkSimulationTime>()
                .write_resource::<EventChannel<NetworkSimulationEvent>>()
                .build(
                    move |_commands, _world, (transport, socket, sim_time, channel), _| {
                        if let Some(socket) = socket.get_mut() {
                            let messages = transport
                                .drain_messages_to_send(|_| sim_time.should_send_message_now());
                            for message in messages {
                                match message.delivery {
                                    DeliveryRequirement::Unreliable
                                    | DeliveryRequirement::Default => {
                                        if let Err(e) =
                                            socket.send_to(&message.payload, message.destination)
                                        {
                                            channel.single_write(
                                                NetworkSimulationEvent::SendError(e, message),
                                            );
                                        }
                                    }
                                    delivery => {
                                        panic!(
                                "{:?} is unsupported. UDP only supports Unreliable by design.",
                                delivery
                            )
                                    }
                                }
                            }
                        }
                    },
                ),
        )
    }
}

/// Creates a new udp network receiver system
pub struct UdpNetworkReceiveSystem;

impl System for UdpNetworkReceiveSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("UdpNetworkReceiveSystem")
                .write_resource::<UdpSocketResource>()
                .write_resource::<EventChannel<NetworkSimulationEvent>>()
                .build(move |_commands, _world, (socket, event_channel), _| {
                    let UdpSocketResource {
                        ref mut socket,
                        ref mut recv_buffer,
                    } = **socket;
                    if let Some(socket) = socket {
                        loop {
                            match socket.recv_from(recv_buffer) {
                                Ok((recv_len, address)) => {
                                    let event = NetworkSimulationEvent::Message(
                                        address,
                                        Bytes::copy_from_slice(&recv_buffer[..recv_len]),
                                    );
                                    // TODO: Handle other types of events.
                                    event_channel.single_write(event);
                                }
                                Err(e) => {
                                    if e.kind() != io::ErrorKind::WouldBlock {
                                        event_channel
                                            .single_write(NetworkSimulationEvent::RecvError(e));
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }),
        )
    }
}

/// Resource to own the UDP socket.
#[derive(Default)]
pub struct UdpSocketResource {
    socket: Option<UdpSocket>,
    recv_buffer: Vec<u8>,
}

impl UdpSocketResource {
    fn new(socket: Option<UdpSocket>, recv_buffer_size_bytes: usize) -> Self {
        Self {
            socket,
            recv_buffer: vec![0; recv_buffer_size_bytes],
        }
    }
}

impl UdpSocketResource {
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
