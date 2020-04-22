//! Network systems implementation backed by the web socket protocol (over TCP).

use tungstenite::{
    client::AutoStream,
    error::Error as TgError,
    handshake::{client::Request, HandshakeError},
};

use amethyst_core::{
    bundle::SystemBundle,
    ecs::{DispatcherBuilder, Read, System, World, Write},
    shrev::EventChannel,
};
use amethyst_error::Error;
use bytes::Bytes;
use log::{error, warn};
use std::{
    collections::HashMap,
    io,
    net::{SocketAddr, TcpListener},
    ops::DerefMut,
};

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

type WebSocketAuto = tungstenite::protocol::WebSocket<AutoStream>;

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
        // NetworkSimulationTime should run first
        // followed by WebSocketConnectionListenerSystem and WebSocketStreamManagementSystem
        // then WebSocketNetworkSendSystem and WebSocketNetworkRecvSystem

        builder.add(
            NetworkSimulationTimeSystem,
            NETWORK_SIM_TIME_SYSTEM_NAME,
            &[],
        );

        builder.add(
            WebSocketConnectionListenerSystem,
            CONNECTION_LISTENER_SYSTEM_NAME,
            &[NETWORK_SIM_TIME_SYSTEM_NAME],
        );

        builder.add(
            WebSocketStreamManagementSystem,
            STREAM_MANAGEMENT_SYSTEM_NAME,
            &[NETWORK_SIM_TIME_SYSTEM_NAME],
        );

        builder.add(
            WebSocketNetworkSendSystem,
            NETWORK_SEND_SYSTEM_NAME,
            &[
                STREAM_MANAGEMENT_SYSTEM_NAME,
                CONNECTION_LISTENER_SYSTEM_NAME,
            ],
        );

        builder.add(
            WebSocketNetworkRecvSystem,
            NETWORK_RECV_SYSTEM_NAME,
            &[
                STREAM_MANAGEMENT_SYSTEM_NAME,
                CONNECTION_LISTENER_SYSTEM_NAME,
            ],
        );

        world.insert(WebSocketNetworkResource::new(self.listener));
        Ok(())
    }
}

/// System to manage the current active WebSocket connections.
pub struct WebSocketStreamManagementSystem;

impl<'s> System<'s> for WebSocketStreamManagementSystem {
    type SystemData = (
        Write<'s, WebSocketNetworkResource>,
        Read<'s, TransportResource>,
        Write<'s, EventChannel<NetworkSimulationEvent>>,
    );

    // We cannot use `web_socket_network_resource.sockets.entry(message.destination)`
    // `.or_insert_with(|| { .. })` because there is a `return;` statement for early exit, which is
    // not allowed within the closure.
    #[allow(clippy::map_entry)]
    fn run(
        &mut self,
        (mut web_socket_network_resource, transport, mut network_simulation_ec): Self::SystemData,
    ) {
        // Make connections for each message in the channel if one hasn't yet been established
        transport.get_messages().iter().for_each(|message| {
            if !web_socket_network_resource
                .sockets
                .contains_key(&message.destination)
            {
                // We are simply establishing a connection for arbitrary data, so we use a blank
                // `Request`.
                //
                // See <https://docs.rs/tungstenite/0.10.1/tungstenite/client/fn.client.html>
                let request = {
                    let uri = format!("ws://{}/", message.destination);
                    Request::builder()
                        .uri(uri)
                        .body(())
                        .expect("Failed to build empty request.")
                };

                // For now, we *do* block on connect, so we don't use the
                // `tungstenite::client::client` method.
                //
                // Ideally we would use that, and if we hit `Err(HandshakeError::Interrupted(_))`,
                // then we would try again later.
                match tungstenite::client::connect(request) {
                    // We don't care about the handshake response
                    Ok((web_socket, _response)) => {
                        dbg!(format!("Connected to {}", &message.destination));
                        web_socket_network_resource
                            .sockets
                            .insert(message.destination, (true, web_socket));
                    }
                    Err(handshake_error) => {
                        let error = io::Error::new(io::ErrorKind::Other, handshake_error);
                        network_simulation_ec.single_write(
                            NetworkSimulationEvent::ConnectionError(
                                error,
                                Some(message.destination),
                            ),
                        );
                        return;
                    }
                }
            }
        });

        // Remove inactive connections
        web_socket_network_resource
            .sockets
            .retain(|addr, (active, _)| {
                if !*active {
                    network_simulation_ec.single_write(NetworkSimulationEvent::Disconnect(*addr));
                }
                *active
            });
    }
}

/// System to listen for incoming connections and cache them to the resource.
pub struct WebSocketConnectionListenerSystem;

impl<'s> System<'s> for WebSocketConnectionListenerSystem {
    type SystemData = (
        Write<'s, WebSocketNetworkResource>,
        Write<'s, EventChannel<NetworkSimulationEvent>>,
    );

    fn run(
        &mut self,
        (mut web_socket_network_resource, mut network_simulation_ec): Self::SystemData,
    ) {
        let resource = web_socket_network_resource.deref_mut();
        if let Some(ref listener) = resource.listener {
            loop {
                match listener.accept() {
                    Ok((stream, addr)) => {
                        stream
                            .set_nonblocking(true)
                            .expect("Setting nonblocking mode");
                        stream.set_nodelay(true).expect("Setting nodelay");

                        match tungstenite::server::accept(AutoStream::Plain(stream)) {
                            Ok(web_socket) => {
                                resource.sockets.insert(addr, (true, web_socket));
                                network_simulation_ec
                                    .single_write(NetworkSimulationEvent::Connect(addr));
                            }
                            Err(HandshakeError::Interrupted(_)) => {
                                break;
                            }
                            Err(HandshakeError::Failure(e)) => {
                                error!("Handshake failure during accept: {}", e);
                                break;
                            }
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        break;
                    }
                    Err(e) => {
                        network_simulation_ec
                            .single_write(NetworkSimulationEvent::ConnectionError(e, None));
                        break;
                    }
                };
            }
        }
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

/// System to receive messages from all open `WebSocket`s.
pub struct WebSocketNetworkRecvSystem;

impl<'s> System<'s> for WebSocketNetworkRecvSystem {
    type SystemData = (
        Write<'s, WebSocketNetworkResource>,
        Write<'s, EventChannel<NetworkSimulationEvent>>,
    );

    fn run(
        &mut self,
        (mut web_socket_network_resource, mut network_simulation_ec): Self::SystemData,
    ) {
        let web_socket_network_resource = web_socket_network_resource.deref_mut();
        for (peer_addr, (active, web_socket)) in web_socket_network_resource.sockets.iter_mut() {
            // If we can't read, the connection may have dropped so we'll mark it inactive.
            let peer_addr = *peer_addr;
            if !web_socket.can_read() {
                warn!("Unable to read from `peer_addr`: `{}`", peer_addr);
                *active = false;
                continue;
            }

            loop {
                match web_socket.read_message() {
                    Ok(message) => {
                        // https://docs.rs/tungstenite/0.10.1/tungstenite/enum.Message.html
                        match message {
                            tungstenite::Message::Text(message_string) => {
                                let event = NetworkSimulationEvent::Message(
                                    peer_addr,
                                    Bytes::copy_from_slice(message_string.as_bytes()),
                                );
                                network_simulation_ec.single_write(event);
                            }
                            tungstenite::Message::Binary(bytes) => {
                                let event = NetworkSimulationEvent::Message(
                                    peer_addr,
                                    Bytes::copy_from_slice(&bytes),
                                );
                                network_simulation_ec.single_write(event);
                            }
                            tungstenite::Message::Ping(_bytes) => {
                                // TODO Send Message::Pong
                            }
                            tungstenite::Message::Pong(bytes) => {
                                // We aren't sending `Ping`s, so shouldn't receive `Pong`s
                                warn!(
                                    "Received `tungstenite::Message::Pong` but reply has not been \
                                    implemented. Bytes: {:?}",
                                    bytes
                                );
                            }
                            tungstenite::Message::Close(_close_frame) => {
                                *active = false;
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        match e {
                            TgError::ConnectionClosed | TgError::AlreadyClosed => {
                                *active = false;
                            }
                            TgError::Io(io_error) => match io_error.kind() {
                                io::ErrorKind::ConnectionReset => *active = false,
                                io::ErrorKind::WouldBlock => {}
                                _ => {
                                    network_simulation_ec
                                        .single_write(NetworkSimulationEvent::RecvError(io_error));
                                }
                            },
                            _ => {
                                let error = io::Error::new(io::ErrorKind::Other, e);
                                network_simulation_ec
                                    .single_write(NetworkSimulationEvent::RecvError(error));
                            }
                        }
                        break;
                    }
                }
            }
        }
    }
}

pub struct WebSocketNetworkResource {
    listener: Option<TcpListener>,
    sockets: HashMap<SocketAddr, (bool, WebSocketAuto)>,
}

impl WebSocketNetworkResource {
    pub fn new(listener: Option<TcpListener>) -> Self {
        Self {
            listener,
            sockets: HashMap::new(),
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
    pub fn get_socket(&mut self, addr: SocketAddr) -> Option<&mut (bool, WebSocketAuto)> {
        self.sockets.get_mut(&addr)
    }

    /// Drops the stream with the given `SocketAddr`. This will be called when a peer seems to have
    /// been disconnected
    pub fn drop_socket(&mut self, addr: SocketAddr) -> Option<(bool, WebSocketAuto)> {
        self.sockets.remove(&addr)
    }
}

impl Default for WebSocketNetworkResource {
    fn default() -> Self {
        Self {
            listener: None,
            sockets: HashMap::new(),
        }
    }
}
