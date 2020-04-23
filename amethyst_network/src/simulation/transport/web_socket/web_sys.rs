use std::{
    collections::HashMap,
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
use log::{debug, error, warn};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Blob, CloseEvent, ErrorEvent, Event, FileReader, MessageEvent, WebSocket};

use crate::simulation::{
    events::NetworkSimulationEvent,
    message::Message,
    requirements::{DeliveryRequirement, UrgencyRequirement},
    timing::NetworkSimulationTime,
    transport::TransportResource,
};

#[wasm_bindgen]
extern "C" {
    fn web_socket_send(web_socket: &WebSocket, src: &[u8]);
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

    fn run(
        &mut self,
        (mut transport, mut web_socket_network_resource, sim_time, mut network_simulation_ec): Self::SystemData,
    ) {
        let messages = transport.drain_messages_to_send(|message| {
            let web_socket_active = web_socket_network_resource
                .streams
                .get(&message.destination)
                .map(|web_socket_and_status| {
                    web_socket_and_status.status == WebSocketStatus::Active
                })
                // `true` because if the message destination does not exist, sendint the message
                // will err, and that will bubble to the user.
                .unwrap_or(true);

            let should_end_now = sim_time.should_send_message_now();

            web_socket_active && should_end_now
        });

        for message in messages {
            match message.delivery {
                DeliveryRequirement::ReliableOrdered(Some(_)) => {
                    warn!("Streams are not supported by `WebSocket`s and will be ignored.");
                    write_message(
                        message,
                        &mut web_socket_network_resource,
                        &mut network_simulation_ec,
                    );
                }
                DeliveryRequirement::ReliableOrdered(_) | DeliveryRequirement::Default => {
                    write_message(
                        message,
                        &mut web_socket_network_resource,
                        &mut network_simulation_ec,
                    );
                }
                delivery => panic!(
                    "{:?} is unsupported. `WebSocket` only supports ReliableOrdered by design.",
                    delivery
                ),
            }
        }
    }
}

fn write_message(
    message: Message,
    web_socket_network_resource: &mut WebSocketNetworkResource,
    _network_simulation_ec: &mut EventChannel<NetworkSimulationEvent>,
) {
    if let Some(WebSocketAndStatus { ref web_socket, .. }) =
        web_socket_network_resource.get_socket(message.destination)
    {
        // We cannot use `AsRef::<[u8]>::as_ref(&message.payload)` because `send_with_u8_array`
        // requires the memory not to be in a `SharedArrayBuffer`.
        // let mut owned_bytes = vec![0; message.payload.len()];
        // owned_bytes.copy_from_slice(AsRef::<[u8]>::as_ref(&message.payload));
        web_socket_send(web_socket, AsRef::<[u8]>::as_ref(&message.payload));
    }
}

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
        let web_socket_network_resource = web_socket_network_resource.deref_mut();
        web_socket_nse_buffer
            .try_iter()
            .for_each(|web_socket_event| {
                let WebSocketEvent {
                    socket_addr,
                    event_type,
                } = web_socket_event;

                match event_type {
                    WebSocketEventType::Open => {
                        // When the `onopen_callback` runs, we set the `WebSocketStatus` to `Active`.
                        let web_socket_and_status =
                            web_socket_network_resource.streams.get_mut(&socket_addr);
                        if let Some(web_socket_and_status) = web_socket_and_status {
                            debug!("`Websocket` `{}` is now `Active`", socket_addr);
                            web_socket_and_status.status = WebSocketStatus::Active
                        } else {
                            warn!(
                                "Received `WebSocketEventType::Open` for `{}`, \
                                but no socket was tracked for that address.",
                                socket_addr
                            );
                        }
                    }
                    WebSocketEventType::Close => {
                        // Even though we could make the `onclose_callback` write
                        // `NetworkSimulationEvent::Disconnect`, we edit the `active` status and allow
                        // the `WebSocketStreamManagementSystem` to send the events, as that keeps the
                        // web implementation consistent with the native implementation.
                        if let Some(WebSocketAndStatus { ref mut status, .. }) =
                            web_socket_network_resource.streams.get_mut(&socket_addr)
                        {
                            *status = WebSocketStatus::Inactive;
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
                match WebSocket::new(&format!("ws://{}/", message.destination)) {
                    Ok(mut web_socket) => {
                        let wse_tx = web_socket_nse_buffer.tx.clone();
                        NseCallbacks::setup(message.destination, &mut web_socket, wse_tx);

                        web_socket_network_resource.streams.insert(
                            message.destination,
                            WebSocketAndStatus {
                                web_socket,
                                status: WebSocketStatus::PendingOpen,
                            },
                        );
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
            .retain(|socket_addr, web_socket_and_status| {
                if web_socket_and_status.status == WebSocketStatus::Inactive {
                    network_simulation_ec
                        .single_write(NetworkSimulationEvent::Disconnect(*socket_addr));
                    false
                } else {
                    true
                }
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

            debug!("Websocket received message: {:?}", data);

            match Blob::instanceof(&data) {
                true => {
                    debug!("Websocket message is blob.");

                    let blob = Blob::unchecked_from_js(data);
                    Self::read_event_data(socket_addr, tx.clone(), blob);
                }
                false => {
                    warn!("Websocket message was not blob: {:?}", data);
                    let error = io::Error::new(io::ErrorKind::Other, format!("{:?}", data));
                    let nse = NetworkSimulationEvent::RecvError(error);
                    let wse = WebSocketEvent {
                        socket_addr,
                        event_type: WebSocketEventType::NetworkSimulationEvent(nse),
                    };
                    tx.send(wse).unwrap_or_else(|error| {
                        error!(
                            "`WebSocket` `onmessage_callback` failed to send event: {:?}",
                            error
                        );
                    });
                }
            }
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
            tx.send(wse).unwrap_or_else(|error| {
                error!(
                    "`WebSocket` `onclose_callback` failed to send event: {:?}",
                    error
                );
            });
        }) as Box<dyn FnMut(CloseEvent)>);
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        let tx = wse_tx.clone();
        let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
            error!(
                "error: {}, file: {}, line: {}, col: {}\n{:?}",
                e.message(),
                e.filename(),
                e.lineno(),
                e.colno(),
                e.error(),
            );

            // Attempt to reconstruct the message from what we know.
            let error = io::Error::new(io::ErrorKind::Other, format!("{:?}", e));
            let payload = {
                let array = Uint8Array::new(&e.error());
                let mut bytes = vec![0; array.length() as usize];
                array.copy_to(&mut bytes);
                bytes
            };
            let delivery = DeliveryRequirement::ReliableOrdered(None);
            let urgency = UrgencyRequirement::OnTick;
            let message = Message::new(socket_addr, &payload, delivery, urgency);
            let nse = NetworkSimulationEvent::SendError(error, message);
            let wse = WebSocketEvent {
                socket_addr,
                event_type: WebSocketEventType::NetworkSimulationEvent(nse),
            };
            tx.send(wse).unwrap_or_else(|error| {
                error!(
                    "`WebSocket` `onmessage_callback` failed to send event: {:?}",
                    error
                );
            });
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        let tx = wse_tx;
        let onopen_callback = Closure::wrap(Box::new(move |_| {
            let wse = WebSocketEvent {
                socket_addr,
                event_type: WebSocketEventType::Open,
            };
            tx.send(wse).unwrap_or_else(|error| {
                error!(
                    "`WebSocket` `onopen_callback` failed to send event: {:?}",
                    error
                );
            });
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
    }

    fn read_event_data(socket_addr: SocketAddr, tx: Sender<WebSocketEvent>, blob: Blob) {
        // `blob` will not contain any data until it is read from by a `FileReader`
        // <https://developer.mozilla.org/en-US/docs/Web/API/FileReader>
        match FileReader::new() {
            Ok(file_reader) => match file_reader.read_as_array_buffer(&blob) {
                Ok(()) => {
                    let tx = tx.clone();
                    let onload_callback = Closure::wrap(Box::new(move |event: Event| {
                        let file_reader: FileReader = event.target().unwrap().dyn_into().unwrap();
                        let array_buffer = file_reader.result().unwrap();

                        let array = Uint8Array::new(AsRef::<JsValue>::as_ref(&array_buffer));
                        if array.length() != 0 {
                            let mut bytes = vec![0; array.length() as usize];
                            array.copy_to(&mut bytes);

                            debug!("Blob bytes: {:?}", bytes);

                            let nse = NetworkSimulationEvent::Message(
                                socket_addr,
                                Bytes::copy_from_slice(&bytes),
                            );
                            let wse = WebSocketEvent {
                                socket_addr,
                                event_type: WebSocketEventType::NetworkSimulationEvent(nse),
                            };
                            tx.send(wse).unwrap_or_else(|error| {
                                error!(
                                    "`WebSocket` `onmessage_callback` failed to send event: {:?}",
                                    error
                                );
                            });
                        } else {
                            debug!("Blob is empty.");
                        }
                    })
                        as Box<dyn FnMut(Event)>);
                    file_reader.set_onload(Some(onload_callback.as_ref().unchecked_ref()));
                    onload_callback.forget();
                }
                Err(e) => {
                    error!("Failed to read blob as array buffer: {:?}", e);
                }
            },
            Err(e) => error!("Failed to construct `FileReader`: {:?}", e),
        }
    }
}

/// Buffer for `NetworkSimulationEvent`s received from a web socket event handler.
///
/// This is an intermediate storage as the event handlers cannot each hold a mutable reference to
/// the `EventChannel<NetworkSimulationEvent>`.
pub struct WebSocketEventBuffer<T> {
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
pub struct WebSocketEvent {
    socket_addr: SocketAddr,
    event_type: WebSocketEventType,
}

#[derive(Debug)]
pub enum WebSocketEventType {
    Open,
    Close,
    NetworkSimulationEvent(NetworkSimulationEvent),
}

pub type WebSocketWseBuffer = WebSocketEventBuffer<WebSocketEvent>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebSocketStatus {
    Inactive,
    PendingOpen,
    Active,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WebSocketAndStatus {
    web_socket: WebSocket,
    status: WebSocketStatus,
}

#[derive(Default)]
pub struct WebSocketNetworkResource {
    streams: HashMap<SocketAddr, WebSocketAndStatus>,
}

impl WebSocketNetworkResource {
    pub fn new() -> Self {
        WebSocketNetworkResource::default()
    }

    /// Returns a tuple of an active WebSocket and whether ot not that stream is active
    pub fn get_socket(&mut self, addr: SocketAddr) -> Option<&mut WebSocketAndStatus> {
        self.streams.get_mut(&addr)
    }

    /// Drops the stream with the given `SocketAddr`. This will be called when a peer seems to have
    /// been disconnected
    pub fn drop_socket(&mut self, addr: SocketAddr) -> Option<WebSocketAndStatus> {
        self.streams.remove(&addr)
    }
}

unsafe impl Send for WebSocketNetworkResource {}
unsafe impl Sync for WebSocketNetworkResource {}
