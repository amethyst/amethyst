//! Network Connection and states.

use shrev::EventChannel;
use amethyst_core::specs::{Component, VecStorage};
use std::net::SocketAddr;
use uuid::Uuid;
use super::NetEvent;

// TODO: Think about relationship between NetConnection and NetIdentity.

/// A network connection target data.
pub struct NetConnection<E> {
    /// The remote socket address of this connection.
    pub target: SocketAddr,
    /// The state of the connection.
    pub state: ConnectionState,
    pub send_buffer: EventChannel<NetEvent<E>>,
    pub receive_buffer: EventChannel<NetEvent<E>>,
}

impl<E: Send+Sync+'static> NetConnection<E>{
  pub fn new(target: SocketAddr) -> Self{
    NetConnection{
      target,
      state: ConnectionState::Connecting,
      send_buffer: EventChannel::<NetEvent<E>>::new(),
      receive_buffer: EventChannel::<NetEvent<E>>::new(),
    }
  }
}

impl<E> PartialEq for NetConnection<E> {
  fn eq(&self, other: &Self) -> bool {
        self.target == other.target && self.state == other.state
    }
}

impl<E: PartialEq> Eq for NetConnection<E> {}

impl<E: Send+Sync+'static> Component for NetConnection<E>{
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

impl Default for NetIdentity{
  fn default() -> Self{
    NetIdentity{
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


