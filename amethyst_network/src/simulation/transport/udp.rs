//! Network systems implementation backed by the UDP network protocol.

use crate::simulation::{
    events::NetworkSimulationEvent,
    requirements::DeliveryRequirement,
    timing::*,
    transport::{
        TransportResource, NETWORK_RECV_SYSTEM_NAME, NETWORK_SEND_SYSTEM_NAME,
        NETWORK_SIM_TIME_SYSTEM_NAME,
    },
};
use amethyst_core::{
    ecs::prelude::*,
    dispatcher::{DispatcherBuilder, Stage, SystemBundle},
    shrev::EventChannel,
};
use amethyst_error::Error;
use bytes::Bytes;
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

impl SystemBundle for UdpNetworkBundle {
    fn build(
        self,
        world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder<'_>,
    ) -> Result<(), Error> {
        builder.add_system(Stage::Begin, build_network_simulation_time_system);
        builder.add_system(Stage::Begin, build_udp_network_recv_system);
        builder.add_system(Stage::Begin, build_udp_network_send_system);

        resources.insert(UdpSocketResource::new(self.socket, self.recv_buffer_size_bytes));
        Ok(())
    }
}


pub fn build_udp_network_send_system(_world: &mut World, _res: &mut Resources) -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("UdpNetworkSendSystem")
        .write_resource::<TransportResource>()
        .write_resource::<UdpSocketResource>()
        .read_resource::<NetworkSimulationTime>()
        .write_resource::<EventChannel<NetworkSimulationEvent>>()
        .build(move |_commands, world, (transport, socket, sim_time, channel), ()| {
            if let (Some(socket), _) = socket.get_mut() {
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

pub fn build_udp_network_recv_system(_world: &mut World, _res: &mut Resources) -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("UdpNetworkRecvSystem")
        .write_resource::<UdpSocketResource>()
        .write_resource::<EventChannel<NetworkSimulationEvent>>()
        .build(move |_commands, world, (socket_res, event_channel), ()| {
            if let (Some(socket), recv_buff) = socket_res.get_mut() {
                loop {
                    match socket.recv_from(recv_buff) {
                        Ok((recv_len, address)) => {
                            let event = NetworkSimulationEvent::Message(
                                address,
                                Bytes::copy_from_slice(&recv_buff[..recv_len]),
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
        Self { 
            socket: None,
            recv_buffer: vec!(),
        }
    }
}

impl UdpSocketResource {
    /// Create a new instance of the `UdpSocketResource`
    pub fn new(socket: Option<UdpSocket>, capacity: usize) -> Self {
        Self { 
            socket, 
            recv_buffer: vec![0; capacity],
        }
    }

    /// Returns a tuple containing an immutable reference to the socket if there is one configured and the associated receive buffer and the associated receive buffer.
    pub fn get(&self) -> (Option<&UdpSocket>, &Vec<u8>) {
        (self.socket.as_ref(), self.recv_buffer.as_ref())
    }

    /// Returns a mutable reference to the socket if there is one configured.
    pub fn get_mut(&mut self) -> (Option<&mut UdpSocket>, &mut Vec<u8>) {
        (self.socket.as_mut(), self.recv_buffer.as_mut())
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
