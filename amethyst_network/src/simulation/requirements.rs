/// Specification of the desired delivery guarantee on a message. All examples will use the
/// following: 1, 2, 3, 4, 5, 6 sent from the server. 5, 1, 4, 2, 3 received by the client. Packet
/// 6 was lost on initial send.

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq)]
pub enum DeliveryRequirement {
    /// Messages may not be delivered.
    /// Client receives 5, 1, 4, 2, 3
    Unreliable,
    /// Messages may not be delivered and the client is guaranteed to only receive the newest
    /// messages.
    /// Client receives 5
    UnreliableSequenced(Option<u8>),
    /// Messages must all be delivered.
    /// Client receives 5, 1, 4, 2, 3, 6
    Reliable,
    /// Messages must all be delivered but only the newest messages are returned to the client.
    /// Client receives 5, 6
    ReliableSequenced(Option<u8>),
    /// Messages must all be delivered and returned to the client in the order they were sent.
    /// This takes an optional "stream_id" which can be used if the underlying transport supports
    /// multiplexed streams. By specifying "None" for the stream_id, the transport can decide
    /// where it wants to put the message.
    /// Client receives 1, 2, 3, 4, 5, 6
    ReliableOrdered(Option<u8>),
    /// Defer to the underlying implementation to decide what "Default" means.
    /// e.g. Udp will have a default delivery of "Unreliable"
    Default,
}

/// Specification of urgency of the sending of a message. Typically we'll want to send messages
/// on simulation tick but the option to send messages immediately is available.
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq)]
pub enum UrgencyRequirement {
    /// Message will be sent based on the current configuration of the simulation frame rate and
    /// the message send rate.
    OnTick,
    /// Message will be sent as soon as possible.
    Immediate,
}
