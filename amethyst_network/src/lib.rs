//! Provides a client-server networking architecture to amethyst.

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

pub use crate::{
    bundle::NetworkBundle,
    connection::{ConnectionState, NetConnection, NetIdentity},
    error::Result,
    net_event::{NetEvent, NetPacket},
    network_socket::NetSocketSystem,
    server::{Host, ServerConfig},
};

use std::net::SocketAddr;

use bincode::{deserialize, serialize};
use crossbeam_channel::Sender;
use laminar::Packet;
use log::error;
use serde::{de::DeserializeOwned, Serialize};

mod bundle;
mod connection;
mod error;
mod net_event;
mod network_socket;
mod server;
mod test;

/// Sends an event to the target NetConnection using the provided network Socket.
/// The socket has to be bound.
pub fn send_event<T>(event: NetPacket<T>, addr: SocketAddr, sender: &Sender<Packet>)
where
    T: Serialize,
{
    let ser = serialize(&event.content());
    match ser {
        Ok(payload) => {
            let packet = match event.delivery_guarantee() {
                net_event::DeliveryGuarantee::Unreliable => match event.ordering_guarantee() {
                    net_event::OrderingGuarantee::None => Packet::unreliable(addr, payload),
                    net_event::OrderingGuarantee::Sequenced(s) => {
                        Packet::unreliable_sequenced(addr, payload, s)
                    }
                    _ => unreachable!(
                        "Can not apply the guarantees: {:?}, {:?} to the packet",
                        event.ordering_guarantee(),
                        event.delivery_guarantee()
                    ),
                },
                net_event::DeliveryGuarantee::Reliable => match event.ordering_guarantee() {
                    net_event::OrderingGuarantee::None => Packet::reliable_unordered(addr, payload),
                    net_event::OrderingGuarantee::Sequenced(s) => {
                        Packet::reliable_sequenced(addr, payload, s)
                    }
                    net_event::OrderingGuarantee::Ordered(o) => {
                        Packet::reliable_ordered(addr, payload, o)
                    }
                },
            };

            match sender.send(packet) {
                Ok(_qty) => {}
                Err(e) => error!("Failed to send data to network socket: {}", e),
            }
        }
        Err(e) => error!("Failed to serialize the event: {}", e),
    }
}

// Attempts to deserialize an event from the raw byte data.
fn deserialize_event<T>(data: &[u8]) -> Result<T>
where
    T: DeserializeOwned,
{
    Ok(deserialize::<T>(data)?)
}
