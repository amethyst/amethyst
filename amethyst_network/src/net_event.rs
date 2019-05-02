//! The network events that are passed from machine to machine, and within the ECS event handling system.
//! NetEvent are passed through the network
//! NetOwnedEvent are passed through the ECS, and contains the event's source (remote connection, usually).

use crate::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::net::SocketAddr;

/// Network events which you can send or and receive from an endpoint.
// TODO, Connect, connection refused, disconnect, disconnected
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NetEvent<T> {
    /// Will be fired when a client connected.
    /// When this event occurs the `NetConnection` with this address was already automatically added to the world.
    Connected(SocketAddr),
    /// Will be fired when a client was disconnected.
    /// If this happens consider removing the `NetConnection` with this address from the world.
    Disconnected(SocketAddr),
    /// Send a packet to all connected clients
    Packet(NetPacket<T>),
    #[doc(hidden)]
    __Nonexhaustive,
}

impl<T> NetEvent<T>
where
    T: Serialize + DeserializeOwned,
{
    pub(crate) fn from_packet(packet: laminar::Packet) -> Result<Self> {
        match crate::deserialize_event::<T>(packet.payload()) {
            Ok(event) => {
                let net_event: NetEvent<T> = NetEvent::Packet(match packet.delivery_guarantee() {
                    laminar::DeliveryGuarantee::Unreliable => match packet.order_guarantee() {
                        laminar::OrderingGuarantee::None => NetPacket::<T>::unreliable(event),
                        laminar::OrderingGuarantee::Sequenced(s) => {
                            NetPacket::unreliable_sequenced(event, s)
                        }
                        _ => panic!("This is in no way possible"),
                    },
                    laminar::DeliveryGuarantee::Reliable => match packet.order_guarantee() {
                        laminar::OrderingGuarantee::None => NetPacket::reliable_unordered(event),
                        laminar::OrderingGuarantee::Sequenced(s) => {
                            NetPacket::reliable_sequenced(event, s)
                        }
                        laminar::OrderingGuarantee::Ordered(o) => {
                            NetPacket::reliable_ordered(event, o)
                        }
                    },
                });

                Ok(net_event)
            }
            Err(e) => Err(e),
        }
    }
}

/// Enum to specify how a packet should be arranged.
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialOrd, PartialEq, Eq)]
pub(crate) enum OrderingGuarantee {
    /// No arranging will be done.
    None,
    /// Packets will be arranged in sequence.
    Sequenced(Option<u8>),
    /// Packets will be arranged in order.
    Ordered(Option<u8>),
}

/// Enum to specify how a packet should be delivered.
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialOrd, PartialEq, Eq)]
pub(crate) enum DeliveryGuarantee {
    /// Packet may or may not be delivered
    Unreliable,
    /// Packet will be delivered
    Reliable,
}

impl From<laminar::OrderingGuarantee> for OrderingGuarantee {
    fn from(ordering: laminar::OrderingGuarantee) -> Self {
        match ordering {
            laminar::OrderingGuarantee::None => OrderingGuarantee::None,
            laminar::OrderingGuarantee::Sequenced(s) => OrderingGuarantee::Sequenced(s),
            laminar::OrderingGuarantee::Ordered(o) => OrderingGuarantee::Ordered(o),
        }
    }
}

impl From<OrderingGuarantee> for laminar::OrderingGuarantee {
    fn from(ordering: OrderingGuarantee) -> Self {
        match ordering {
            OrderingGuarantee::None => laminar::OrderingGuarantee::None,
            OrderingGuarantee::Sequenced(s) => laminar::OrderingGuarantee::Sequenced(s),
            OrderingGuarantee::Ordered(o) => laminar::OrderingGuarantee::Ordered(o),
        }
    }
}

impl From<DeliveryGuarantee> for laminar::DeliveryGuarantee {
    fn from(delivery: DeliveryGuarantee) -> Self {
        match delivery {
            DeliveryGuarantee::Unreliable => laminar::DeliveryGuarantee::Unreliable,
            DeliveryGuarantee::Reliable => laminar::DeliveryGuarantee::Reliable,
        }
    }
}

impl Default for OrderingGuarantee {
    fn default() -> Self {
        OrderingGuarantee::None
    }
}

impl Default for DeliveryGuarantee {
    fn default() -> Self {
        DeliveryGuarantee::Unreliable
    }
}

/// Represents a packet which could have any serializable payload.
///
/// A packet could have reliability guarantees to specify how it should be delivered and processed.
///
/// | Reliability Type                 | Packet Drop | Packet Duplication | Packet Order  | Packet Fragmentation |Packet Delivery|
/// | :-------------:                  | :-------------: | :-------------:    | :-------------:  | :-------------:  | :-------------:
/// |       **Unreliable Unordered**   |       Yes       |       Yes          |      No          |      No          |       No
/// |       **Unreliable Sequenced**   |       Yes       |      No            |      Sequenced   |      No          |       No
/// |       **Reliable Unordered**     |       No        |      No            |      No          |      Yes         |       Yes
/// |       **Reliable Ordered**       |       No        |      No            |      Ordered     |      Yes         |       Yes
/// |       **Reliable Sequenced**     |       No        |      No            |      Sequenced     |      Yes       |       Yes
///
/// You are able to send packets with any the above guarantees.
///
/// For more information please have a look at: https://amethyst.github.io/laminar/docs/reliability/reliability.html
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetPacket<T> {
    content: T,
    #[serde(skip)]
    ordering_guarantee: OrderingGuarantee,
    #[serde(skip)]
    delivery_guarantee: DeliveryGuarantee,
}

impl<T> NetPacket<T> {
    /// Create a new unreliable packet with the given content.
    ///
    /// Unreliable: Packets can be dropped, duplicated or arrive without order.
    ///
    /// **Details**
    ///
    /// | Packet Drop     | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
    /// | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
    /// |       Yes       |        Yes         |      No          |      No              |       No        |
    ///
    /// Basically just bare UDP. The packet may or may not be delivered.
    pub fn unreliable(content: T) -> NetPacket<T> {
        NetPacket {
            ordering_guarantee: OrderingGuarantee::None,
            delivery_guarantee: DeliveryGuarantee::Unreliable,
            content,
        }
    }

    /// Create a new unreliable sequenced packet with the given content.
    ///
    /// Unreliable Sequenced; Packets can be dropped, but could not be duplicated and arrive in sequence.
    ///
    /// *Details*
    ///
    /// | Packet Drop     | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
    /// | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
    /// |       Yes       |        Yes         |      Sequenced          |      No              |       No  |
    ///
    /// Basically just bare UDP, free to be dropped, but has some sequencing to it so that only the newest packets are kept.
    pub fn unreliable_sequenced(content: T, stream_id: Option<u8>) -> NetPacket<T> {
        NetPacket {
            ordering_guarantee: OrderingGuarantee::Sequenced(stream_id),
            delivery_guarantee: DeliveryGuarantee::Unreliable,
            content,
        }
    }

    /// Create a new packet with the given content.
    /// Reliable; All packets will be sent and received, but without order.
    ///
    /// *Details*
    ///
    /// |   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
    /// | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
    /// |       No        |      No            |      No          |      Yes             |       Yes       |
    ///
    /// Basically this is almost TCP without ordering of packets.
    pub fn reliable_unordered(content: T) -> NetPacket<T> {
        NetPacket {
            ordering_guarantee: OrderingGuarantee::None,
            delivery_guarantee: DeliveryGuarantee::Reliable,
            content,
        }
    }

    /// Create a new packet with the given content and optional stream on which the ordering will be done.
    ///
    /// Reliable; All packets will be sent and received, with order.
    ///
    /// *Details*
    ///
    /// |   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
    /// | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
    /// |       No        |      No            |      Ordered     |      Yes             |       Yes       |
    ///
    /// Basically this is almost TCP-like with ordering of packets.
    ///
    /// # Remark
    /// - When `stream_id` is specified as `None` the default stream will be used; if you are not sure what this is you can leave it at `None`.
    pub fn reliable_ordered(content: T, stream_id: Option<u8>) -> NetPacket<T> {
        NetPacket {
            ordering_guarantee: OrderingGuarantee::Ordered(stream_id),
            delivery_guarantee: DeliveryGuarantee::Reliable,
            content,
        }
    }

    /// Create a new packet with the given content and optional stream on which the sequencing will be done.
    ///
    /// Reliable; All packets will be sent and received, but arranged in sequence.
    /// Which means that only the newest packets will be let through, older packets will be received but they won't get to the user.
    ///
    /// *Details*
    ///
    /// |   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
    /// | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
    /// |       Yes        |      No            |      Sequenced     |      Yes             |       Yes       |
    ///
    /// Basically this is almost TCP-like but then sequencing instead of ordering.
    ///
    /// # Remark
    /// - When `stream_id` is specified as `None` the default stream will be used; if you are not sure what this is you can leave it at `None`.
    pub fn reliable_sequenced(content: T, stream_id: Option<u8>) -> NetPacket<T> {
        NetPacket {
            ordering_guarantee: OrderingGuarantee::Sequenced(stream_id),
            delivery_guarantee: DeliveryGuarantee::Reliable,
            content,
        }
    }

    /// Returns if this event is reliable.
    ///
    /// Each net event type is either reliable or unreliable.
    /// Reliable events always reach their destination, unreliable events may be lost.
    pub fn is_reliable(&self) -> bool {
        self.delivery_guarantee == DeliveryGuarantee::Reliable
    }

    /// Returns if this event is unreliable.
    ///
    /// Each net event type is either reliable or unreliable.
    /// Reliable events always reach their destination, unreliable events may be lost.
    pub fn is_unreliable(&self) -> bool {
        self.delivery_guarantee == DeliveryGuarantee::Unreliable
    }

    /// Returns whether this event is an ordered event.
    pub fn is_ordered(&self) -> bool {
        if let OrderingGuarantee::Ordered(_) = self.ordering_guarantee {
            return true;
        }
        false
    }

    /// Returns whether this event is an sequenced event.
    pub fn is_sequenced(&self) -> bool {
        if let OrderingGuarantee::Sequenced(_) = self.ordering_guarantee {
            return true;
        }
        false
    }

    /// Return if this event is neither ordered or sequenced.
    pub fn is_unordered(&self) -> bool {
        self.ordering_guarantee == OrderingGuarantee::None
    }

    /// Returns a immutable reference to the content.
    pub fn content(&self) -> &T {
        &self.content
    }

    /// Returns a immutable reference to the content.
    pub fn content_mut(&mut self) -> &mut T {
        &mut self.content
    }

    /// Returns the ordering guarantee
    pub(crate) fn ordering_guarantee(&self) -> OrderingGuarantee {
        self.ordering_guarantee
    }

    /// Returns the delivery guarantee
    pub(crate) fn delivery_guarantee(&self) -> DeliveryGuarantee {
        self.delivery_guarantee
    }
}

#[cfg(test)]
mod tests {
    use crate::net_event::NetPacket;

    #[test]
    fn assure_creation_unreliable_packet() {
        let packet = NetPacket::unreliable(test_payload());

        assert_eq!(packet.content(), &test_payload());
        assert_eq!(packet.is_ordered(), false);
        assert_eq!(packet.is_sequenced(), false);
        assert_eq!(packet.is_reliable(), false);
        assert_eq!(packet.is_unreliable(), true);
    }

    #[test]
    fn assure_creation_unreliable_sequenced() {
        let packet = NetPacket::unreliable_sequenced(test_payload(), Some(1));

        assert_eq!(packet.content(), &test_payload());
        assert_eq!(packet.is_ordered(), false);
        assert_eq!(packet.is_sequenced(), true);
        assert_eq!(packet.is_reliable(), false);
        assert_eq!(packet.is_unreliable(), true);
    }

    #[test]
    fn assure_creation_reliable() {
        let packet = NetPacket::reliable_unordered(test_payload());

        assert_eq!(packet.content(), &test_payload());
        assert_eq!(packet.is_ordered(), false);
        assert_eq!(packet.is_sequenced(), false);
        assert_eq!(packet.is_reliable(), true);
        assert_eq!(packet.is_unreliable(), false);
    }

    #[test]
    fn assure_creation_reliable_ordered() {
        let packet = NetPacket::reliable_ordered(test_payload(), Some(1));

        assert_eq!(packet.content(), &test_payload());
        assert_eq!(packet.is_ordered(), true);
        assert_eq!(packet.is_sequenced(), false);
        assert_eq!(packet.is_reliable(), true);
        assert_eq!(packet.is_unreliable(), false);
    }

    #[test]
    fn assure_creation_reliable_sequence() {
        let packet = NetPacket::reliable_sequenced(test_payload(), Some(1));

        assert_eq!(packet.content(), &test_payload());
        assert_eq!(packet.is_ordered(), false);
        assert_eq!(packet.is_sequenced(), true);
        assert_eq!(packet.is_reliable(), true);
        assert_eq!(packet.is_unreliable(), false);
    }

    fn test_payload() -> Vec<u8> {
        return "test".as_bytes().to_vec();
    }
}
