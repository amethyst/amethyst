//! Provides a client-server networking architecture to amethyst.

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

pub use crate::{
    bundle::NetworkBundle,
    connection::{ConnectionState, NetConnection, NetIdentity},
    error::Result,
    filter::{FilterConnected, NetFilter},
    net_event::NetEvent,
    network_socket::NetSocketSystem,
    server::{Host, ServerConfig, ServerSocketEvent},
};

use std::{net::SocketAddr, sync::mpsc::SyncSender};

use bincode::{deserialize, serialize};
use laminar::Packet;
use log::error;
use serde::{de::DeserializeOwned, Serialize};

mod bundle;
mod connection;
mod error;
mod filter;
mod net_event;
mod network_socket;
mod server;
mod test;

/// Each event type is either reliable or unreliable:
/// Reliable events always reach their destination,
/// Unreliable events may be lost
/// For Amethyst-defined events, whether it's reliable is specified in this function,
/// Otherwise, it's specified by the use of NetEvent::Reliable vs NetEvent::Unreliable
fn is_reliable<T>(event: NetEvent<T>) -> bool {
	use NetEvent as NE;
	match event {
		// I specify them all explicitly so the typechecker can save
		// us from the mistake of specifying a builtin that SHOULD be
		// unreliable, but is assumed to be unreliable like all the rest
		| NE::Connect {..}
		| NE::Connected {..}
		| NE::ConnectionRefused {..}
		| NE::Disconnect {..}
		| NE::Disconnected {..}
		| NE::TextMessage {..}
		| NE::Reliable(_)
		=> true,
		| NE::Unreliable(_)
		=> false,
	}
}

/// Sends an event to the target NetConnection using the provided network Socket.
/// The socket has to be bound.
pub fn send_event<T>(event: NetEvent<T>, addr: SocketAddr, sender: &SyncSender<ServerSocketEvent>)
where
    T: Serialize,
{
    let ser = serialize(&event);
    match ser {
        Ok(s) => {
            let p = if is_reliable(event) {
	            Packet::unreliable(addr, s)
            } else {
	            Packet::reliable_unordered(addr, s)
            };
            match sender.send(ServerSocketEvent::Packet(p)) {
                Ok(_qty) => {}
                Err(e) => error!("Failed to send data to network socket: {}", e),
            }
        }
        Err(e) => error!("Failed to serialize the event: {}", e),
    }
}

// Attempts to deserialize an event from the raw byte data.
fn deserialize_event<T>(data: &[u8]) -> Result<NetEvent<T>>
where
    T: DeserializeOwned,
{
    Ok(deserialize::<NetEvent<T>>(data)?)
}
