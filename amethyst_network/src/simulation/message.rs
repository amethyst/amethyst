use super::requirements::{DeliveryRequirement, UrgencyRequirement};
use bytes::Bytes;

/// Structure used to hold message payloads before they are consumed and sent by an underlying
/// NetworkSystem.
#[derive(Debug, PartialEq, Eq)]
pub struct Message {
    /// The serialized payload itself.
    pub(crate) payload: Bytes,
    /// The requirement around whether or not this message should be resent if lost.
    pub(crate) delivery: DeliveryRequirement,
    /// The requirement around when this message should be sent.
    pub(crate) urgency: UrgencyRequirement,
}

impl Message {
    /// Creates and returns a new Message.
    pub(crate) fn new(
        payload: &[u8],
        delivery: DeliveryRequirement,
        urgency: UrgencyRequirement,
    ) -> Self {
        Self {
            payload: Bytes::from(payload),
            delivery,
            urgency,
        }
    }
}
