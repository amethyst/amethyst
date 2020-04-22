use std::{io, net::TcpStream, ops::DerefMut};

use amethyst_core::{
    ecs::{Read, System, Write},
    shrev::EventChannel,
};
use bytes::Bytes;
use log::{error, warn};
use tungstenite::{
    error::Error as TgError,
    handshake::{client::Request, HandshakeError},
};

use crate::simulation::{events::NetworkSimulationEvent, transport::TransportResource};

use super::WebSocketNetworkResource;

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
                let mut handshake_result = tungstenite::client::client(request, stream);
                loop {
                    match handshake_result {
                        Err(HandshakeError::Interrupted(client_handshake)) => {
                            // This is the expected result when connecting with a non-blocking TCP stream.
                            // Next, we start the client_handshake and try to get its final result.
                            handshake_result = client_handshake.handshake();
                        }
                        // We don't care about the handshake response
                        Ok((web_socket, _response)) => {
                            dbg!(format!("Connected to {}", &message.destination));
                            web_socket_network_resource
                                .streams
                                .insert(message.destination, (true, web_socket));
                            break;
                        }
                        Err(HandshakeError::Failure(TgError::Io(io_error))) => {
                            network_simulation_ec.single_write(
                                NetworkSimulationEvent::ConnectionError(
                                    io_error,
                                    Some(message.destination),
                                ),
                            );
                            break;
                        }
                        Err(handshake_error) => {
                            let error = io::Error::new(io::ErrorKind::Other, handshake_error);
                            network_simulation_ec.single_write(
                                NetworkSimulationEvent::ConnectionError(
                                    error,
                                    Some(message.destination),
                                ),
                            );
                            break;
                        }
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
