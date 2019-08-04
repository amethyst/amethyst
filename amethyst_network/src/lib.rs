//! Provides a client-server networking architecture to amethyst.

#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

#[doc(no_inline)]
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
use laminar::Packet;
use serde::{de::DeserializeOwned, Serialize};

mod bundle;
mod connection;
mod error;
mod net_event;
mod network_socket;
mod server;
mod test;

/// Attempts to serialize the given `NetEvent` and returns a laminar packet.
/// Reliable ordered will be used by default.
fn serialize_event<E>(event: NetEvent<E>, addr: SocketAddr) -> Result<Packet>
where
    E: Serialize,
{
    match serialize(&event) {
        Ok(packet) => Ok(Packet::reliable_ordered(addr, packet, None)),
        Err(e) => Err(e.into()),
    }
}

/// Attempts to serialize the given packet and returns a laminar packet.
fn serialize_packet<T>(packet: NetPacket<T>, addr: SocketAddr) -> Result<Packet>
where
    T: Serialize,
{
    let ser = serialize(&packet.content());
    match ser {
        Ok(payload) => Ok(match packet.delivery_guarantee() {
            net_event::DeliveryGuarantee::Unreliable => match packet.ordering_guarantee() {
                net_event::OrderingGuarantee::None => Packet::unreliable(addr, payload),
                net_event::OrderingGuarantee::Sequenced(s) => {
                    Packet::unreliable_sequenced(addr, payload, s)
                }
                _ => unreachable!(
                    "Can not apply the guarantees: {:?}, {:?} to the packet.",
                    packet.ordering_guarantee(),
                    packet.delivery_guarantee()
                ),
            },
            net_event::DeliveryGuarantee::Reliable => match packet.ordering_guarantee() {
                net_event::OrderingGuarantee::None => Packet::reliable_unordered(addr, payload),
                net_event::OrderingGuarantee::Sequenced(s) => {
                    Packet::reliable_sequenced(addr, payload, s)
                }
                net_event::OrderingGuarantee::Ordered(o) => {
                    Packet::reliable_ordered(addr, payload, o)
                }
            },
        }),
        Err(e) => Err(e.into()),
    }
}

// Attempts to deserialize an event from the raw byte data.
fn deserialize_event<T>(data: &[u8]) -> Result<T>
where
    T: DeserializeOwned,
{
    Ok(deserialize::<T>(data)?)
}

#[cfg(test)]
mod tests {
    use crate::{deserialize_event, net_event::NetPacket, serialize_packet};
    use laminar::{DeliveryGuarantee, OrderingGuarantee};
    use std::net::SocketAddr;

    #[test]
    fn can_serialize_packets() {
        let content = "abc".to_string();
        let packet1 = NetPacket::reliable_unordered(content.clone());
        let packet2 = NetPacket::reliable_ordered(content.clone(), None);
        let packet3 = NetPacket::reliable_sequenced(content.clone(), None);
        let packet4 = NetPacket::unreliable(content.clone());
        let packet5 = NetPacket::unreliable_sequenced(content.clone(), None);

        let addr: SocketAddr = "127.0.0.1:1234".parse().unwrap();

        let serialized_packet1 = serialize_packet(packet1, addr).unwrap();
        let serialized_packet2 = serialize_packet(packet2, addr).unwrap();
        let serialized_packet3 = serialize_packet(packet3, addr).unwrap();
        let serialized_packet4 = serialize_packet(packet4, addr).unwrap();
        let serialized_packet5 = serialize_packet(packet5, addr).unwrap();

        // assure correct guarantees
        assert!(
            serialized_packet1.delivery_guarantee() == DeliveryGuarantee::Reliable
                && serialized_packet1.order_guarantee() == OrderingGuarantee::None
        );
        assert!(
            serialized_packet2.delivery_guarantee() == DeliveryGuarantee::Reliable
                && serialized_packet2.order_guarantee() == OrderingGuarantee::Ordered(None)
        );
        assert!(
            serialized_packet3.delivery_guarantee() == DeliveryGuarantee::Reliable
                && serialized_packet3.order_guarantee() == OrderingGuarantee::Sequenced(None)
        );
        assert!(
            serialized_packet4.delivery_guarantee() == DeliveryGuarantee::Unreliable
                && serialized_packet4.order_guarantee() == OrderingGuarantee::None
        );
        assert!(
            serialized_packet5.delivery_guarantee() == DeliveryGuarantee::Unreliable
                && serialized_packet5.order_guarantee() == OrderingGuarantee::Sequenced(None)
        );
    }

    #[test]
    fn can_deserialize_event() {
        let result =
            deserialize_event::<NetPacket<String>>(&[3, 0, 0, 0, 0, 0, 0, 0, 97, 98, 99]).unwrap();

        assert_eq!(result.content(), &"abc".to_string());
    }
}
