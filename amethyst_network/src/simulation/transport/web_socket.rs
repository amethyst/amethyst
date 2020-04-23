//! Network systems implementation backed by the web socket protocol (over TCP).

#[cfg(target_arch = "x86_64")]
mod native;
#[cfg(target_arch = "x86_64")]
pub use self::native::{
    WebSocketConnectionListenerSystem, WebSocketNetworkRecvSystem, WebSocketNetworkResource,
    WebSocketNetworkSendSystem, WebSocketStreamManagementSystem,
};

#[cfg(target_arch = "x86_64")]
use std::net::TcpListener;

#[cfg(target_arch = "wasm32")]
mod web_sys;
#[cfg(target_arch = "wasm32")]
pub use self::web_sys::{
    WebSocketNetworkRecvSystem, WebSocketNetworkResource, WebSocketNetworkSendSystem,
    WebSocketStreamManagementSystemDesc,
};

#[cfg(target_arch = "wasm32")]
use amethyst_core::SystemDesc;
use amethyst_core::{
    bundle::SystemBundle,
    ecs::{DispatcherBuilder, World},
};
use amethyst_error::Error;

use crate::simulation::{
    timing::NetworkSimulationTimeSystem,
    transport::{NETWORK_RECV_SYSTEM_NAME, NETWORK_SEND_SYSTEM_NAME, NETWORK_SIM_TIME_SYSTEM_NAME},
};

#[cfg(target_arch = "x86_64")]
const CONNECTION_LISTENER_SYSTEM_NAME: &str = "ws_connection_listener";
const STREAM_MANAGEMENT_SYSTEM_NAME: &str = "ws_stream_management";

/// Use this network bundle to add the `WebSocket` transport layer to your game.
#[cfg(target_arch = "x86_64")]
pub struct WebSocketNetworkBundle {
    listener: Option<TcpListener>,
}

/// Use this network bundle to add the `WebSocket` transport layer to your game.
#[cfg(target_arch = "wasm32")]
pub struct WebSocketNetworkBundle;

impl WebSocketNetworkBundle {
    #[cfg(target_arch = "x86_64")]
    pub fn new(listener: Option<TcpListener>) -> Self {
        Self { listener }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new() -> Self {
        Self
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for WebSocketNetworkBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'_, '_>,
    ) -> Result<(), Error> {
        // `NetworkSimulationTime` should run first
        // followed by `WebSocketConnectionListenerSystem` (if present) and
        // `WebSocketStreamManagementSystem`
        // then `WebSocketNetworkSendSystem` and `WebSocketNetworkRecvSystem`

        builder.add(
            NetworkSimulationTimeSystem,
            NETWORK_SIM_TIME_SYSTEM_NAME,
            &[],
        );

        // `wasm32` targets are not allowed to listen for connections.
        #[cfg(target_arch = "x86_64")]
        builder.add(
            WebSocketConnectionListenerSystem,
            CONNECTION_LISTENER_SYSTEM_NAME,
            &[NETWORK_SIM_TIME_SYSTEM_NAME],
        );

        #[cfg(target_arch = "x86_64")]
        let (web_socket_stream_management_system, send_recv_deps) = {
            (
                WebSocketStreamManagementSystem,
                &[
                    STREAM_MANAGEMENT_SYSTEM_NAME,
                    CONNECTION_LISTENER_SYSTEM_NAME,
                ],
            )
        };
        #[cfg(target_arch = "wasm32")]
        let (web_socket_stream_management_system, send_recv_deps) = {
            (
                WebSocketStreamManagementSystemDesc::default().build(world),
                &[STREAM_MANAGEMENT_SYSTEM_NAME],
            )
        };

        builder.add(
            web_socket_stream_management_system,
            STREAM_MANAGEMENT_SYSTEM_NAME,
            &[NETWORK_SIM_TIME_SYSTEM_NAME],
        );

        builder.add(
            WebSocketNetworkSendSystem,
            NETWORK_SEND_SYSTEM_NAME,
            send_recv_deps,
        );

        builder.add(
            WebSocketNetworkRecvSystem,
            NETWORK_RECV_SYSTEM_NAME,
            send_recv_deps,
        );

        #[cfg(target_arch = "x86_64")]
        world.insert(WebSocketNetworkResource::new(self.listener));
        #[cfg(target_arch = "wasm32")]
        world.insert(WebSocketNetworkResource::new());
        Ok(())
    }
}
