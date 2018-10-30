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

pub use {
    bundle::NetworkBundle,
    connection::{ConnectionState, NetConnection, NetIdentity},
    filter::{FilterConnected, NetFilter},
    net_event::NetEvent,
    network_socket::NetSocketSystem,
};

use std::net::{SocketAddr, UdpSocket};

use bincode::{deserialize, serialize, ErrorKind};
use serde::{de::DeserializeOwned, Serialize};

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
