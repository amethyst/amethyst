//! Provides a client-server networking architecture to amethyst.

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

mod bundle;
mod connection;
mod filter;
mod net_event;
mod network_socket;
mod test;

pub use crate::{
    bundle::NetworkBundle,
    connection::{ConnectionState, NetConnection, NetIdentity},
    filter::{FilterConnected, NetFilter},
    net_event::NetEvent,
    network_socket::NetSocketSystem,
};

use std::net::SocketAddr;

use bincode::{deserialize, serialize, ErrorKind};
use laminar::net::UdpSocket;
use laminar::Packet;
use serde::{de::DeserializeOwned, Serialize};

/// Sends an event to the target NetConnection using the provided network Socket.
/// The socket has to be bound.
pub fn send_event<T>(event: &NetEvent<T>, addr: &SocketAddr, socket: &mut UdpSocket)
where
    T: Serialize,
{
    let ser = serialize(event);
    match ser {
        Ok(s) => {
            let slice = s.as_slice();
            // send an unreliable `Packet` from laminar which is basically just a bare UDP packet.
            match socket.send(&Packet::unreliable(*addr, slice.to_vec())) {
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
