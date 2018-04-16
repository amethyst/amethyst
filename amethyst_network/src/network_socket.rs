//! The network send and receive System

use super::{deserialize_event, send_event, ConnectionState, NetConnection, NetConnectionPool,
            NetEvent, NetFilter, NetReceiveBuffer, NetSendBuffer, NetSourcedEvent};
use mio::{Events, Poll, PollOpt, Ready, Token};
use mio::net::UdpSocket;
use serde::Serialize;
use serde::de::DeserializeOwned;
use shrev::*;
use specs::{Fetch, FetchMut, System};
use std::clone::Clone;
use std::io::Error;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::str;
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

const SOCKET: Token = Token(0);

// If a client sends both a connect event and other events,
// only the connect event will be considered valid and all others will be lost.

/// The System managing the network state and connections.
/// The T generic parameter corresponds to the network event enum type.
/// Receives events and filters them.
/// Received events will be inserted into the NetReceiveBuffer resource.
/// To send an event, add it to the NetSendBuffer resource.
///
/// If both a connection (Connect or Connected) event is received at the same time as another event from the same connection,
/// only the connection event will be considered and rest will be filtered out.
// TODO: add Unchecked Event type list. Those events will be let pass the client connected filter (Example: NetEvent::Connect).
// Current behaviour: hardcoded passthrough of Connect and Connected events.
pub struct NetSocketSystem<T>
where
    T: PartialEq,
{
    /// The network socket.
    pub socket: UdpSocket,
    /// The reader for the NetSendBuffer.
    pub send_queue_reader: Option<ReaderId<NetSourcedEvent<T>>>,
    /// The list of filters applied on the events received.
    pub filters: Vec<Box<NetFilter<T>>>,
    /// The mio's `Poll`.
    pub poll: Poll,
}

impl<T> NetSocketSystem<T>
where
    T: Serialize + PartialEq,
{
    /// Creates a NetClientSystem and binds the Socket on the ip and port added in parameters.
    pub fn new(
        ip: &str,
        port: u16,
        filters: Vec<Box<NetFilter<T>>>,
    ) -> Result<NetSocketSystem<T>, Error> {
        let socket = UdpSocket::bind(&SocketAddr::new(
            IpAddr::from_str(ip).expect("Unreadable input IP."),
            port,
        ))?;
        let poll = Poll::new()?;
        poll.register(&socket, SOCKET, Ready::readable(), PollOpt::level())?;
        Ok(NetSocketSystem {
            socket,
            send_queue_reader: None,
            filters,
            poll,
        })
    }
    /// Connects to a remote server (client-only call)
    pub fn connect(&mut self, target: SocketAddr, pool: &mut NetConnectionPool, client_uuid: Uuid) {
        info!("Sending connection request to remote {:?}", target);
        let conn = NetConnection {
            target,
            state: ConnectionState::Connecting,
            uuid: None,
        };
        send_event(&NetEvent::Connect::<T> { client_uuid }, &conn, &self.socket);
        pool.connections.push(conn);
    }
}

impl<'a, T> System<'a> for NetSocketSystem<T>
where
    T: Send + Sync + Serialize + Clone + DeserializeOwned + PartialEq + 'static,
{
    type SystemData = (
        FetchMut<'a, NetSendBuffer<T>>,
        FetchMut<'a, NetReceiveBuffer<T>>,
        Fetch<'a, NetConnectionPool>,
    );
    fn run(&mut self, (mut send_buf, mut receive_buf, pool): Self::SystemData) {
        let mut events = Events::with_capacity(2048);
        let mut buf = [0 as u8; 2048];

        // Sends events that are in the NetSendBuffer resource.
        if self.send_queue_reader.is_none() {
            self.send_queue_reader = Some(send_buf.buf.register_reader());
        }

        for ev in send_buf.buf.read(self.send_queue_reader.as_mut().unwrap()) {
            let target = pool.connection_from_address(&ev.socket);
            if let Some(t) = target {
                if t.state == ConnectionState::Connected || t.state == ConnectionState::Connecting {
                    send_event(&ev.event, &t, &self.socket);
                } else {
                    warn!("Tried to send packet while target is not in a connected or connecting state.");
                }
            } else {
                warn!("Targeted address is not in the NetConnection pool.")
            }
        }

        // Receives event through mio's `Poll`.
        loop {
            self.poll
                .poll(&mut events, Some(Duration::from_millis(0)))
                .expect("Failed to poll network socket.");

            if events.is_empty() {
                break;
            }

            for raw_event in events.iter() {
                if raw_event.readiness().is_readable() {
                    match self.socket.recv_from(&mut buf) {
                        // Data received
                        Ok((amt, src)) => {
                            let net_event = deserialize_event::<T>(&buf[..amt]);
                            match net_event {
                                Ok(ev) => {
                                    // Filter events
                                    let mut filtered = false;
                                    for mut f in self.filters.iter_mut() {
                                        if !f.allow(&pool, &src, &ev) {
                                            filtered = true;
                                        }
                                    }
                                    if !filtered {
                                        let owned_event = NetSourcedEvent {
                                            event: ev.clone(),
                                            uuid: pool.connection_from_address(&src)
                                                .and_then(|c| c.uuid),
                                            socket: src,
                                        };
                                        receive_buf.buf.single_write(owned_event);
                                    } else {
                                        info!("Filtered an incoming network packet.")
                                    }
                                }
                                Err(e) => error!("Failed to read network event: {}", e),
                            }
                        }
                        Err(e) => {
                            error!("Could not receive datagram: {}", e);
                        }
                    }
                }
            }
        }
    }
}
