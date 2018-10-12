//! Provides a client-server networking architecture to amethyst.

#![warn(missing_docs)]

extern crate amethyst_core;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
extern crate bincode;
extern crate fern;
extern crate shred;
extern crate shrev;
extern crate uuid;

mod bundle;
mod connection;
mod filter;
mod net_event;
mod network_socket;
mod test;

pub use bundle::NetworkBundle;
pub use connection::{ConnectionState, NetConnection, NetIdentity};
pub use filter::{FilterConnected, NetFilter};
pub use net_event::NetEvent;
pub use network_socket::NetSocketSystem;

use bincode::ErrorKind;
use bincode::{deserialize, serialize};

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::net::SocketAddr;
use std::net::UdpSocket;

/// Sends an event to the target NetConnection using the provided network Socket.
/// The socket has to be bound.
pub fn send_event<T>(event: &NetEvent<T>, target: &SocketAddr, socket: &UdpSocket)
where
    T: Serialize,
{
    let ser = serialize(event);
    match ser {
        Ok(s) => {
            let slice = s.as_slice();
            match socket.send_to(slice, target) {
                Ok(_qty) => {}
                Err(e) => error!("Failed to send data to network socket: {}", e),
            }
        }
        Err(e) => error!("Failed to serialize the event: {}", e),
    }
}

/// Attempts to deserialize an event from the raw byte data.
fn deserialize_event<T>(data: &[u8]) -> Result<NetEvent<T>, Box<ErrorKind>>
where
    T: DeserializeOwned,
{
    deserialize::<NetEvent<T>>(data)
}
