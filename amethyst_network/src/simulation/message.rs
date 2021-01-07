use std::net::SocketAddr;

use bytes::Bytes;

use super::requirements::{DeliveryRequirement, UrgencyRequirement};

/// Structure used to hold message payloads before they are consumed and sent by an underlying
/// NetworkSystem.
#[derive(Debug, PartialEq, Eq)]
pub struct Message {
    /// The destination to send the message.
    pub destination: SocketAddr,
    /// The serialized payload itself.
    pub payload: Bytes,
    /// The requirement around whether or not this message should be resent if lost.
    pub delivery: DeliveryRequirement,
    /// The requirement around when this message should be sent.
    pub urgency: UrgencyRequirement,
}

impl Message {
    /// Creates and returns a new Message.
    pub(crate) fn new(
        destination: SocketAddr,
        payload: &[u8],
        delivery: DeliveryRequirement,
        urgency: UrgencyRequirement,
    ) -> Self {
        Self {
            destination,
            payload: Bytes::copy_from_slice(payload),
            delivery,
            urgency,
        }
    }
}
