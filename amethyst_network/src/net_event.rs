//! The network events that are passed from machine to machine, and within the ECS event handling system.
//! NetEvent are passed through the network
//! NetOwnedEvent are passed through the ECS, and contains the event's source (remote connection, usually).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The basic network events shipped with amethyst.
// TODO: Add CreateEntity,RemoveEntity,UpdateEntity once specs 0.11 is released
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NetEvent<T> {
    /// Ask to connect to the server.
    Connect {
        /// The client uuid.
        client_uuid: Uuid,
    },
    /// Reply to the client that the connection has been accepted.
    Connected {
        /// The server uuid.
        server_uuid: Uuid,
    },
    /// Reply to the client that the connection has been refused.
    ConnectionRefused {
        /// The reason of the refusal.
        reason: String,
    },
    /// Tell the server that the client is disconnecting.
    Disconnect {
        /// The reason of the disconnection.
        reason: String,
    },
    /// Notify the clients(including the one being disconnected) that a client has been disconnected from the server.
    Disconnected {
        /// The reason of the disconnection.
        reason: String,
    },
    /// A simple text message event.
    TextMessage {
        /// The message.
        msg: String,
    },
    /// There are two user-defined types containing more network event types:
    /// Reliable events will keep sending until the target confirms receipt
    Reliable(T),
    /// Unreliable events will send a bare packet, whether lost or not
    Unreliable(T),
}

impl<T> NetEvent<T> {
    /// Tries to convert a NetEvent to a custom event type.
    pub fn custom(&self) -> Option<&T> {
        match self {
            NetEvent::Reliable(ref t) | NetEvent::Unreliable(ref t) => Some(&t),
            _ => None,
        }
    }
	/// Each event type is either reliable or unreliable:
	/// Reliable events always reach their destination,
	/// Unreliable events may be lost
	/// For Amethyst-defined events, whether it's reliable is specified in this function,
	/// Otherwise, it's specified by the use of NetEvent::Reliable vs NetEvent::Unreliable
	pub fn is_reliable(&self) -> bool {
	    use NetEvent as NE;
	    match self {
	        // I specify them all explicitly so the typechecker can save
	        // us from the mistake of specifying a builtin that SHOULD be
	        // unreliable, but is assumed to be unreliable like all the rest
	        NE::Connect { .. }
	        | NE::Connected { .. }
	        | NE::ConnectionRefused { .. }
	        | NE::Disconnect { .. }
	        | NE::Disconnected { .. }
	        | NE::TextMessage { .. }
	        | NE::Reliable(_) => true,
	        NE::Unreliable(_) => false,
	    }
	}
}
