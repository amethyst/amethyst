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

/// Send the given packet onto the given sender.
///
/// This function serializes the data of the given packet and converts it to a laminar packet.
/// After this, it will be queued on the given `Sender`.
pub fn send_event<T>(packet: NetPacket<T>, addr: SocketAddr, sender: &Sender<Packet>)
where
    T: Serialize,
{
    match serialize_event(packet, addr) {
        Ok(packet) => match sender.send(packet) {
            Ok(_qty) => {}
            Err(e) => error!("Failed to send data to network socket: {}", e),
        },
        Err(e) => error!("Cannot serialize packet. Reason: {}", e),
    };
}

/// Attempts to serialize the given packet and returns a laminar packet.
fn serialize_event<T>(packet: NetPacket<T>, addr: SocketAddr) -> Result<Packet>
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
    use crate::net_event::{NetEvent, NetPacket};
    use crate::{deserialize_event, send_event, serialize_event};
    use crossbeam_channel::unbounded;
    use laminar::{DeliveryGuarantee, OrderingGuarantee};
    use std::net::SocketAddr;
    use std::sync::mpsc::Sender;

    #[test]
    fn can_serialize_packets() {
        let content = "abc".to_string();
        let packet1 = NetPacket::reliable_unordered(content.clone());
        let packet2 = NetPacket::reliable_ordered(content.clone(), None);
        let packet3 = NetPacket::reliable_sequenced(content.clone(), None);
        let packet4 = NetPacket::unreliable(content.clone());
        let packet5 = NetPacket::unreliable_sequenced(content.clone(), None);

        let addr: SocketAddr = "127.0.0.1:1234".parse().unwrap();

        let serialized_packet1 = serialize_event(packet1, addr).unwrap();
        let serialized_packet2 = serialize_event(packet2, addr).unwrap();
        let serialized_packet3 = serialize_event(packet3, addr).unwrap();
        let serialized_packet4 = serialize_event(packet4, addr).unwrap();
        let serialized_packet5 = serialize_event(packet5, addr).unwrap();

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

    #[test]
    fn packet_is_queued_for_send() {
        let packet = NetPacket::reliable_unordered("a".to_string());
        let (tx, rx) = unbounded();
        send_event(packet, "127.0.0.1:0".parse().unwrap(), &tx);

        assert_eq!(rx.len(), 1);
    }
}
