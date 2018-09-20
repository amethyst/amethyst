//! Network Connection and states.

use super::NetEvent;
use amethyst_core::specs::{Component, VecStorage};
use shrev::{EventChannel, EventIterator, ReaderId};
use std::net::SocketAddr;
use uuid::Uuid;

// TODO: Think about relationship between NetConnection and NetIdentity.

/// A network connection target data.
pub struct NetConnection<E: 'static> {
    /// The remote socket address of this connection.
    pub target: SocketAddr,
    /// The state of the connection.
    pub state: ConnectionState,
    pub send_buffer: EventChannel<NetEvent<E>>,
    pub receive_buffer: EventChannel<NetEvent<E>>,
    /// Private. Used by `NetSocketSystem` to be able to immediately send events upon receiving a new NetConnection.
    send_reader: ReaderId<NetEvent<E>>,
}

impl<E: Send + Sync + 'static> NetConnection<E> {
    pub fn new(target: SocketAddr) -> Self {
        let mut send_buffer = EventChannel::new();
        let send_reader = send_buffer.register_reader();
        NetConnection {
            target,
            state: ConnectionState::Connecting,
            send_buffer,
            receive_buffer: EventChannel::<NetEvent<E>>::new(),
            send_reader,
        }
    }

    /// Function used ONLY by NetSocketSystem.
    /// Since most users will want to both create the connection and send messages on the same frame,
    /// we need a way to read those. Since the NetSocketSystem runs after the creation of the NetConnection,
    /// it cannot possibly have registered his reader early enough to catch the initial messages that the user wants to send.
    ///
    /// The downside of this is that you are forced to take NetConnection mutably inside of NetSocketSystem.
    /// If someone finds a better solution, please open a PR.
    pub fn send_buffer_early_read(&mut self) -> EventIterator<NetEvent<E>> {
        self.send_buffer.read(&mut self.send_reader)
    }
}

impl<E> PartialEq for NetConnection<E> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target && self.state == other.state
    }
}

impl<E: PartialEq> Eq for NetConnection<E> {}

impl<E: Send + Sync + 'static> Component for NetConnection<E> {
    type Storage = VecStorage<Self>;
}

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

impl Default for NetIdentity {
    fn default() -> Self {
        NetIdentity {
            uuid: Uuid::new_v4(),
        }
    }
}

impl Component for NetIdentity {
    type Storage = VecStorage<NetIdentity>;
}

/*
/// The list of network connections allocated by the network system.
#[derive(Default)]
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
        self.connections.iter().filter(|c| c.target == *socket).next()
    }

    /// Fetches the NetConnection from the network socket address mutably.
    pub fn connection_from_address_mut(
        &mut self,
        socket: &SocketAddr,
    ) -> Option<&mut NetConnection> {
        self.connections.iter_mut().filter(|c| c.target == *socket).next()
    }

    /// Removes the connection for the specified network socket address.
    pub fn remove_connection_for_address(&mut self, socket: &SocketAddr) -> Option<NetConnection> {
        let index = self.connections.iter().position(|c| c.target == *socket);
        index.map(|i| self.connections.swap_remove(i))
    }
}

impl Component for NetConnectionPool {
    type Storage = VecStorage<NetConnectionPool>;
}*/
