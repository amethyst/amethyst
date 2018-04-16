//! Network Connection and states.

use super::NetSourcedEvent;
use shrev::EventChannel;
use specs::{Component, VecStorage};
use std::net::SocketAddr;
use uuid::Uuid;

/// A network connection target data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetConnection {
    /// The remote socket address of this connection.
    pub target: SocketAddr,
    /// The state of the connection.
    pub state: ConnectionState,
    /// UUID of the owner at the endpoint of this connection.
    /// Will be none during the connection phase of the client, until the server acknowledges the connection.
    pub uuid: Option<Uuid>,
}

impl Eq for NetConnection {}

///The state of the connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// The connection is established.
    Connected,
    /// The connection is being established.
    Connecting,
    /// The connection has been dropped.
    Disconnected,
}

/// A network identity. It can represent either a client or a server.
/// It represents anything that can own an entity or a component.
/// Think of it as an identity card.
/// When used as a resource, it designates the local network uuid.
pub struct NetIdentity {
    /// The uuid identifying this NetIdentity.
    pub uuid: Uuid,
}

impl Component for NetIdentity {
    type Storage = VecStorage<NetIdentity>;
}

/// The list of network connections allocated by the network system.
pub struct NetConnectionPool {
    /// The connections.
    pub connections: Vec<NetConnection>,
}

impl NetConnectionPool {
    /// Creates a new NetConnectionPool.
    pub fn new() -> Self {
        NetConnectionPool {
            connections: vec![],
        }
    }

    /// Fetches the NetConnection from the uuid.
    pub fn connection_from_uuid(&self, uuid: &Uuid) -> Option<&NetConnection> {
        for c in &self.connections {
            if let Some(cl_uuid) = c.uuid {
                if cl_uuid == *uuid {
                    return Some(c);
                }
            }
        }
        None
    }

    /// Fetches the NetConnection from the network socket address.
    pub fn connection_from_address(&self, socket: &SocketAddr) -> Option<&NetConnection> {
        for c in &self.connections {
            if c.target == *socket {
                return Some(c);
            }
        }
        None
    }

    /// Fetches the NetConnection from the network socket address mutably.
    pub fn connection_from_address_mut(&mut self, socket: &SocketAddr) -> Option<&mut NetConnection> {
        for c in &mut self.connections.iter_mut() {
            if c.target == *socket {
                return Some(c);
            }
        }
        None
    }

    /// Removes the connection for the specified network socket address.
    pub fn remove_connection_for_address(&mut self, socket: &SocketAddr) -> Option<NetConnection> {
        let index = self.connections.iter().position(|c| c.target == *socket);
        index.map(|i| self.connections.swap_remove(i))
    }
}

impl Component for NetConnectionPool {
    type Storage = VecStorage<NetConnectionPool>;
}

/// The resource containing the events that should be send by the NetworkSocketSystem.
pub struct NetSendBuffer<T> {
    /// The event buffer.
    pub buf: EventChannel<NetSourcedEvent<T>>,
}

impl<T> NetSendBuffer<T>
where
    T: Send + Sync + 'static,
{
    /// Creates a new empty NetSendBuffer
    pub fn new() -> NetSendBuffer<T> {
        NetSendBuffer {
            buf: EventChannel::<NetSourcedEvent<T>>::new(),
        }
    }
}

impl<T> Component for NetSendBuffer<T>
where
    T: Send + Sync + 'static,
{
    type Storage = VecStorage<NetSendBuffer<T>>;
}

/// The resource containing the events that were received and not filtered by the NetworkSocketSystem.
pub struct NetReceiveBuffer<T> {
    /// The event buffer.
    pub buf: EventChannel<NetSourcedEvent<T>>,
}

impl<T> NetReceiveBuffer<T>
where
    T: Send + Sync + 'static,
{
    /// Creates a new empty NetSendBuffer
    pub fn new() -> NetReceiveBuffer<T> {
        NetReceiveBuffer {
            buf: EventChannel::<NetSourcedEvent<T>>::new(),
        }
    }
}

impl<T> Component for NetReceiveBuffer<T>
where
    T: Send + Sync + 'static,
{
    type Storage = VecStorage<NetReceiveBuffer<T>>;
}
