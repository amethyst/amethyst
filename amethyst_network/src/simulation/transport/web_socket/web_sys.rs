use std::{
    io,
    net::SocketAddr,
    ops::{Deref, DerefMut},
};

use amethyst_core::{
    ecs::{Read, ReadExpect, System, SystemData, World, Write},
    shrev::EventChannel,
    SystemDesc,
};
use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use js_sys::Uint8Array;
use log::error;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{CloseEvent, ErrorEvent, Event, MessageEvent, WebSocket};

use crate::simulation::{events::NetworkSimulationEvent, transport::TransportResource};

use super::WebSocketNetworkResource;

/// System to receive messages from all open `WebSocket`s.
pub struct WebSocketNetworkRecvSystem;

impl<'s> System<'s> for WebSocketNetworkRecvSystem {
    type SystemData = (
        Write<'s, WebSocketNetworkResource>,
        Write<'s, EventChannel<NetworkSimulationEvent>>,
        ReadExpect<'s, WebSocketWseBuffer>,
    );

    fn run(
        &mut self,
        (mut web_socket_network_resource, mut network_simulation_ec, web_socket_nse_buffer): Self::SystemData,
    ) {
        // Transfer all `NetworkSimulationEvent`s from the `WebSocketWseBuffer` into the
        // `EventChannel<NetworkSimulationEvent>`.
        web_socket_nse_buffer
            .try_iter()
            .for_each(|web_socket_event| {
                let WebSocketEvent {
                    socket_addr,
                    event_type,
                } = web_socket_event;

                match event_type {
                    WebSocketEventType::Close => {
                        // Even though we could make the `onclose_callback` write
                        // `NetworkSimulationEvent::Disconnect`, we edit the `active` status and allow
                        // the `WebSocketStreamManagementSystem` to send the events, as that keeps the
                        // web implementation consistent with the native implementation.
                        if let Some((active, _web_socket)) = web_socket_network_resource
                            .deref_mut()
                            .streams
                            .get_mut(&socket_addr)
                        {
                            *active = false;
                        }
                    }
                    WebSocketEventType::NetworkSimulationEvent(nse) => {
                        network_simulation_ec.single_write(nse);
                    }
                }
            });
    }
}

/// Builds a `WebSocketStreamManagementSystem`.
#[derive(Default, Debug)]
pub struct WebSocketStreamManagementSystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, WebSocketStreamManagementSystem>
    for WebSocketStreamManagementSystemDesc
{
    fn build(self, world: &mut World) -> WebSocketStreamManagementSystem {
        <WebSocketStreamManagementSystem as System<'_>>::SystemData::setup(world);

        let (tx, rx) = crossbeam_channel::unbounded();
        let web_socket_nse_buffer = WebSocketWseBuffer { tx, rx };
        world.insert(web_socket_nse_buffer);

        WebSocketStreamManagementSystem
    }
}

/// System to manage the current active WebSocket connections.
pub struct WebSocketStreamManagementSystem;

impl<'s> System<'s> for WebSocketStreamManagementSystem {
    type SystemData = (
        Write<'s, WebSocketNetworkResource>,
        Read<'s, TransportResource>,
        Write<'s, EventChannel<NetworkSimulationEvent>>,
        ReadExpect<'s, WebSocketWseBuffer>,
    );

    // We cannot use `web_socket_network_resource.streams.entry(message.destination)`
    // `.or_insert_with(|| { .. })` because there is a `return;` statement for early exit, which is
    // not allowed within the closure.
    #[allow(clippy::map_entry)]
    fn run(
        &mut self,
        (
            mut web_socket_network_resource,
            transport,
            mut network_simulation_ec,
            web_socket_nse_buffer,
        ): Self::SystemData,
    ) {
        // Make connections for each message in the channel if one hasn't yet been established
        transport.get_messages().iter().for_each(|message| {
            if !web_socket_network_resource
                .streams
                .contains_key(&message.destination)
            {
                // Create a WebSocket
                match WebSocket::new("wss://echo.websocket.org") {
                    Ok(ws) => {
                        let wse_tx = web_socket_nse_buffer.tx.clone();
                        NseCallbacks::setup(message.destination, &mut ws, wse_tx);

                        web_socket_network_resource
                            .streams
                            .insert(message.destination, (true, ws));
                    }
                    Err(e) => {
                        let event = Event::from(e);
                        let error = io::Error::new(io::ErrorKind::Other, event.type_());
                        network_simulation_ec.single_write(
                            NetworkSimulationEvent::ConnectionError(
                                error,
                                Some(message.destination),
                            ),
                        );
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

/// `NetworkSimulationEvent` callbacks for `WebSocket`s.
struct NseCallbacks;

impl NseCallbacks {
    /// Sets up the web socket to send `NetworkSimulationEvent`s.
    pub fn setup(socket_addr: SocketAddr, ws: &mut WebSocket, wse_tx: Sender<WebSocketEvent>) {
        let tx = wse_tx.clone();
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            // Convert javascript ArrayBuffer into Vec<u8>
            let data = e.data();
            let array = Uint8Array::new(&data);
            let mut bytes = Vec::with_capacity(array.length() as usize);
            array.copy_to(&mut bytes);

            let nse = NetworkSimulationEvent::Message(peer_addr, Bytes::copy_from_slice(&bytes));
            let wse = WebSocketEvent {
                socket_addr,
                event_type: WebSocketEventType::NetworkSimulationEvent(nse),
            };
            tx.send(wse);
        }) as Box<dyn FnMut(MessageEvent)>);

        // Set message event handler on `WebSocket`.
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        // Forget the callback to keep it alive.
        onmessage_callback.forget();

        let tx = wse_tx.clone();
        let onclose_callback = Closure::wrap(Box::new(move |e: CloseEvent| {
            // If we receive an `onclose`, we are disconnected from the server, which is likely an
            // error.
            error!("Disconnected from server: {}", e.reason());

            let wse = WebSocketEvent {
                socket_addr,
                event_type: WebSocketEventType::Close,
            };
            tx.send(wse);
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
            error!(
                "error: {}, file: {}, line: {}, col: {}\n{:?}",
                e.message(),
                e.filename(),
                e.lineno(),
                e.colno(),
                e.error(),
            );

            // TODO: send event?
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();
    }
}

/// Buffer for `NetworkSimulationEvent`s received from a web socket event handler.
///
/// This is an intermediate storage as the event handlers cannot each hold a mutable reference to
/// the `EventChannel<NetworkSimulationEvent>`.
struct WebSocketEventBuffer<T> {
    /// Sender for `T` events.
    tx: Sender<T>,
    /// Receiver for `T` events.
    rx: Receiver<T>,
}

impl<T> Deref for WebSocketEventBuffer<T> {
    type Target = Receiver<T>;

    fn deref(&self) -> &Self::Target {
        &self.rx
    }
}

impl<T> DerefMut for WebSocketEventBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rx
    }
}

#[derive(Debug)]
struct WebSocketEvent {
    socket_addr: SocketAddr,
    event_type: WebSocketEventType,
}

#[derive(Debug)]
enum WebSocketEventType {
    Close,
    NetworkSimulationEvent(NetworkSimulationEvent),
}

type WebSocketWseBuffer = WebSocketEventBuffer<WebSocketEvent>;
