//! Network systems implementation backed by the web socket protocol (over TCP).

#[cfg(target_arch = "x86_64")]
mod native;
#[cfg(target_arch = "x86_64")]
use self::native::{
    WebSocketConnectionListenerSystem, WebSocketNetworkRecvSystem, WebSocketStreamManagementSystem,
};

#[cfg(target_arch = "x86_64")]
use std::net::TcpStream;
use std::{
    collections::HashMap,
    io,
    net::{SocketAddr, TcpListener},
};

#[cfg(target_arch = "wasm32")]
mod web_sys;
#[cfg(target_arch = "wasm32")]
use self::web_sys::{WebSocketNetworkRecvSystem, WebSocketStreamManagementSystemDesc};

#[cfg(target_arch = "wasm32")]
use amethyst_core::SystemDesc;
use amethyst_core::{
    bundle::SystemBundle,
    ecs::{DispatcherBuilder, Read, System, World, Write},
    shrev::EventChannel,
};
use amethyst_error::Error;
use log::warn;

use crate::simulation::{
    events::NetworkSimulationEvent,
    message::Message,
    requirements::DeliveryRequirement,
    timing::{NetworkSimulationTime, NetworkSimulationTimeSystem},
    transport::{
        TransportResource, NETWORK_RECV_SYSTEM_NAME, NETWORK_SEND_SYSTEM_NAME,
        NETWORK_SIM_TIME_SYSTEM_NAME,
    },
};

#[cfg(target_arch = "x86_64")]
type WebSocket = tungstenite::protocol::WebSocket<TcpStream>;
#[cfg(target_arch = "wasm32")]
type WebSocket = ::web_sys::WebSocket;

#[cfg(target_arch = "x86_64")]
const CONNECTION_LISTENER_SYSTEM_NAME: &str = "ws_connection_listener";
const STREAM_MANAGEMENT_SYSTEM_NAME: &str = "ws_stream_management";

/// Use this network bundle to add the TCP transport layer to your game.
pub struct WebSocketNetworkBundle {
    listener: Option<TcpListener>,
}

impl WebSocketNetworkBundle {
    pub fn new(listener: Option<TcpListener>) -> Self {
        Self { listener }
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

        world.insert(WebSocketNetworkResource::new(self.listener));
        Ok(())
    }
}

/// System to send messages to a particular open `WebSocket`.
pub struct WebSocketNetworkSendSystem;

impl<'s> System<'s> for WebSocketNetworkSendSystem {
    type SystemData = (
        Write<'s, TransportResource>,
        Write<'s, WebSocketNetworkResource>,
        Read<'s, NetworkSimulationTime>,
        Write<'s, EventChannel<NetworkSimulationEvent>>,
    );

    fn run(&mut self, (mut transport, mut net, sim_time, mut channel): Self::SystemData) {
        let messages = transport.drain_messages_to_send(|_| sim_time.should_send_message_now());
        for message in messages {
            match message.delivery {
                DeliveryRequirement::ReliableOrdered(Some(_)) => {
                    warn!("Streams are not supported by TCP and will be ignored.");
                    write_message(message, &mut net, &mut channel);
                }
                DeliveryRequirement::ReliableOrdered(_) | DeliveryRequirement::Default => {
                    write_message(message, &mut net, &mut channel);
                }
                delivery => panic!(
                    "{:?} is unsupported. TCP only supports ReliableOrdered by design.",
                    delivery
                ),
            }
        }
    }
}

fn write_message(
    message: Message,
    net: &mut WebSocketNetworkResource,
    channel: &mut EventChannel<NetworkSimulationEvent>,
) {
    if let Some((_, web_socket)) = net.get_socket(message.destination) {
        if let Err(e) =
            web_socket.write_message(tungstenite::Message::Binary(message.payload.to_vec()))
        {
            let error = io::Error::new(io::ErrorKind::Other, e);
            channel.single_write(NetworkSimulationEvent::SendError(error, message));
        }
    }
}

pub struct WebSocketNetworkResource {
    listener: Option<TcpListener>,
    streams: HashMap<SocketAddr, (bool, WebSocket)>,
}

impl WebSocketNetworkResource {
    pub fn new(listener: Option<TcpListener>) -> Self {
        Self {
            listener,
            streams: HashMap::new(),
        }
    }

    /// Returns an immutable reference to the listener if there is one configured.
    pub fn get(&self) -> Option<&TcpListener> {
        self.listener.as_ref()
    }

    /// Returns a mutable reference to the listener if there is one configured.
    pub fn get_mut(&mut self) -> Option<&mut TcpListener> {
        self.listener.as_mut()
    }

    /// Sets the bound listener to the `WebSocketNetworkResource`.
    pub fn set_listener(&mut self, listener: TcpListener) {
        self.listener = Some(listener);
    }

    /// Drops the listener from the `WebSocketNetworkResource`.
    pub fn drop_listener(&mut self) {
        self.listener = None;
    }

    /// Returns a tuple of an active WebSocket and whether ot not that stream is active
    pub fn get_socket(&mut self, addr: SocketAddr) -> Option<&mut (bool, WebSocket)> {
        self.streams.get_mut(&addr)
    }

    /// Drops the stream with the given `SocketAddr`. This will be called when a peer seems to have
    /// been disconnected
    pub fn drop_socket(&mut self, addr: SocketAddr) -> Option<(bool, WebSocket)> {
        self.streams.remove(&addr)
    }
}

impl Default for WebSocketNetworkResource {
    fn default() -> Self {
        Self {
            listener: None,
            streams: HashMap::new(),
        }
    }
}

// TODO: Split implementation for `WebSocketNetworkResource`
#[cfg(target_arch = "wasm32")]
unsafe impl Send for WebSocketNetworkResource {}
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for WebSocketNetworkResource {}
