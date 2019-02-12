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

/// Sends an event to the target NetConnection using the provided network Socket.
/// The socket has to be bound.
pub fn send_event<T>(event: NetEvent<T>, addr: SocketAddr, sender: &SyncSender<ServerSocketEvent>)
where
    T: Serialize,
{
    let ser = serialize(&event);
    match ser {
        Ok(s) => {
            let slice = s.as_slice();
            // send an unreliable `Packet` from laminar which is basically just a bare UDP packet.
            match sender.send(ServerSocketEvent::Packet(Packet::unreliable(
                addr,
                slice.to_owned(),
            ))) {
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
