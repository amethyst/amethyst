//! The network events that are passed from machine to machine, and within the ECS event handling system.
//! NetEvent are passed through the network
//! NetOwnedEvent are passed through the ECS, and contains the event's source (remote connection, usually).

use std::net::SocketAddr;
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
    /// A user-defined enum containing more network event types.
    Custom(T),
}

impl<T> NetEvent<T> {
    /// Tries to convert a NetEvent to a custom event enum type.
    pub fn custom(&self) -> Option<&T> {
        if let &NetEvent::Custom(ref t) = self {
            Some(&t)
        } else {
            None
        }
    }
}

///Carries the source of the event. Useful for debugging, security checks, gameplay logic, etc...
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetSourcedEvent<T> {
    /// The event.
    pub event: NetEvent<T>,
    /// The source of this event.
    /// Might be none if the client is connecting.
    pub uuid: Option<Uuid>,
    /// The socket from which we received this event or to which we want to send it to.
    pub socket: SocketAddr,
}
