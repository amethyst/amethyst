//! Network systems implementation backed by the TCP network protocol.

use crate::simulation::{
    timing::{NetworkSimulationTime, NetworkSimulationTimeSystem},
    transport::{NETWORK_RECV_SYSTEM_NAME, NETWORK_SEND_SYSTEM_NAME, NETWORK_SIM_TIME_SYSTEM_NAME},
};
use amethyst_core::{
    bundle::SystemBundle,
    ecs::{DispatcherBuilder, Read, System, World, Write},
    shrev::EventChannel,
};
use amethyst_error::Error;

const CONNECTION_LISTENER_SYSTEM_NAME: &str = "connection_listener";

/// Use this network bundle to add the various underlying tcp network systems to your game.
pub struct TcpNetworkBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for TcpNetworkBundle {
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
        builder.add(
            TcpConnectionListenerSystem,
            CONNECTION_LISTENER_SYSTEM_NAME,
            &[NETWORK_SIM_TIME_SYSTEM_NAME],
        );
        builder.add(
            TcpNetworkSendSystem,
            NETWORK_SEND_SYSTEM_NAME,
            &[CONNECTION_LISTENER_SYSTEM_NAME],
        );
        builder.add(
            TcpNetworkRecvSystem,
            NETWORK_RECV_SYSTEM_NAME,
            &[CONNECTION_LISTENER_SYSTEM_NAME],
        );
        Ok(())
    }
}

/// System to listen for incoming connections and cache them to the resource.
pub struct TcpConnectionListenerSystem;

impl<'s> System<'s> for TcpConnectionListenerSystem {
    type SystemData = (
        Write<'s, TcpNetworkResource>,
        Read<'s, NetworkSimulationTime>,
    );

    fn run(&mut self, data: Self::SystemData) {}
}

/// System to send messages to a particular open `TcpStream`.
pub struct TcpNetworkSendSystem;

impl<'s> System<'s> for TcpNetworkSendSystem {
    type SystemData = ();

    fn run(&mut self, data: Self::SystemData) {}
}

/// System to receive messages from all open `TcpStream`s.
pub struct TcpNetworkRecvSystem;

impl<'s> System<'s> for TcpNetworkRecvSystem {
    type SystemData = ();

    fn run(&mut self, data: Self::SystemData) {}
}

pub struct TcpNetworkResource;

impl Default for TcpNetworkResource {
    fn default() -> Self {
        Self {}
    }
}
