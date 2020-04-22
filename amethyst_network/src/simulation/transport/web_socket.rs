//! Network systems implementation backed by the web socket protocol (over TCP).

use tungstenite::{
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
    net::{SocketAddr, TcpListener, TcpStream},
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

type WebSocketTcp = tungstenite::protocol::WebSocket<TcpStream>;

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

    // We cannot use `web_socket_network_resource.streams.entry(message.destination)`
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
                .streams
                .contains_key(&message.destination)
            {
                let stream = match TcpStream::connect(message.destination) {
                    Ok(stream) => stream,
                    Err(e) => {
                        network_simulation_ec.single_write(
                            NetworkSimulationEvent::ConnectionError(e, Some(message.destination)),
                        );
                        return;
                    }
                };
                stream
                    .set_nonblocking(true)
                    .expect("Setting non-blocking mode");
                stream.set_nodelay(true).expect("Setting nodelay");

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
                match tungstenite::client::client(request, stream) {
                    // We don't care about the handshake response
                    Ok((web_socket, _response)) => {
                        dbg!(format!("Connected to {}", &message.destination));
                        web_socket_network_resource
                            .streams
                            .insert(message.destination, (true, web_socket));
                    }
                    Err(HandshakeError::Interrupted(_)) => {
                        // TODO: retry connecting.
                    }
                    Err(HandshakeError::Failure(TgError::Io(io_error))) => {
                        match io_error.kind() {
                            io::ErrorKind::WouldBlock => {
                                // TODO: retry connecting
                            }
                            _ => {
                                network_simulation_ec.single_write(
                                    NetworkSimulationEvent::ConnectionError(
                                        io_error,
                                        Some(message.destination),
                                    ),
                                );
                            }
                        }
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
            .streams
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

                        match tungstenite::server::accept(stream) {
                            Ok(web_socket) => {
                                resource.streams.insert(addr, (true, web_socket));
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
        for (_, (active, web_socket_tcp)) in web_socket_network_resource.streams.iter_mut() {
            // If we can't get a peer_addr, there is likely something pretty wrong with the
            // connection so we'll mark it inactive.
            let peer_addr = match web_socket_tcp.get_ref().peer_addr() {
                Ok(addr) => addr,
                Err(e) => {
                    warn!("Encountered an error getting peer_addr: {:?}", e);
                    *active = false;
                    continue;
                }
            };

            loop {
                match web_socket_tcp.read_message() {
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
    streams: HashMap<SocketAddr, (bool, WebSocketTcp)>,
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
    pub fn get_socket(&mut self, addr: SocketAddr) -> Option<&mut (bool, WebSocketTcp)> {
        self.streams.get_mut(&addr)
    }

    /// Drops the stream with the given `SocketAddr`. This will be called when a peer seems to have
    /// been disconnected
    pub fn drop_socket(&mut self, addr: SocketAddr) -> Option<(bool, WebSocketTcp)> {
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
