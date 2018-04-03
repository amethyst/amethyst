//! Network Connection and states.

use super::{NetEvent, NetSourcedEvent};
use shrev::EventChannel;
use specs::{Component, VecStorage};
use std::net::SocketAddr;
use uuid::Uuid;

/// A network connection target data.
//TODO add ping here?
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
    pub fn new() -> Self {
        NetConnectionPool {
            connections: vec![],
        }
    }

    pub fn connection_from_uuid(&self, uuid: &Uuid) -> Option<NetConnection> {
        for c in &self.connections {
            if let Some(cl_uuid) = c.uuid {
                if cl_uuid == *uuid {
                    return Some(c.clone());
                }
            }
        }
        None
    }

    pub fn connection_from_address(&self, socket: &SocketAddr) -> Option<NetConnection> {
        for c in &self.connections {
            if c.target == *socket {
                return Some(c.clone());
            }
        }
        None
    }

    pub fn remove_connection_for_address(&mut self, socket: &SocketAddr) -> Option<NetConnection> {
        let index = self.connections.iter().position(|c| c.target == *socket);
        index.map(|i| self.connections.swap_remove(i))
    }
}

impl Component for NetConnectionPool {
    type Storage = VecStorage<NetConnectionPool>;
}

pub struct NetSendBuffer<T> {
    pub buf: EventChannel<NetSourcedEvent<T>>,
}

impl<T> NetSendBuffer<T>
where
    T: Send + Sync + 'static,
{
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

pub struct NetReceiveBuffer<T> {
    pub buf: EventChannel<NetSourcedEvent<T>>,
}

impl<T> NetReceiveBuffer<T>
where
    T: Send + Sync + 'static,
{
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

/*
Client:
1 NetConnection->Server
* NetOwner->Other players + server

Server:
* NetConnection->Clients
* NetOwner (partial)->Clients

NetOwner->uuid->stuff(name,stats,PlayerIdentity,etc...)
NetConnection->socket


SELECT TARGET
send_to_all // nothing special
send_to_others // Need to know self
send_to_all_except // Need to select a target
send_to // Reply to event
send_event // Internal logic

Kick playername = "bob"
playername->PlayerIdentity->netowneruuid    /*->NetOwner->uuid*/  ->NetConnection //works on server but not client

NetOwner->uuid == NetConnection->uuid

UUID assigned by server. Can be fetched from web service using connection token sent from client
*/
