//! Network Connection and states.

use std::net::SocketAddr;
use specs::{Component,VecStorage};
use shrev::EventChannel;
use uuid::Uuid;
use super::{NetEvent,NetSourcedEvent};

/// A network connection target data.
//TODO add ping here?
#[derive(Debug,Clone,Serialize,Deserialize,PartialEq)]
pub struct NetConnection{
    /// The remote socket address of this connection.
    pub target: SocketAddr,
    /// The state of the connection.
    pub state: ConnectionState,
    /// UUID of the owner at the endpoint of this connection.
    pub uuid: Uuid,
}

impl Eq for NetConnection{}

///The state of the connection.
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub enum ConnectionState{
    /// The connection is established.
    Connected,
    /// The connection is being established.
    Connecting,
    /// The connection has been dropped.
    Disconnected,
}

/// A network owner. It can be either the client or a server.
/// It represents anything that can own an entity or a component.
/// Think of it as an identity card.
pub struct NetOwner{
    pub id: Uuid,
}

/// The list of network connections allocated by the network system.
pub struct NetConnectionPool{
    /// The connections.
    pub connections: Vec<NetConnection>,
}

impl NetConnectionPool{
    pub fn new() -> Self{
        NetConnectionPool{
            connections: vec![],
        }
    }

    pub fn connection_from_uuid(&self,uuid:&Uuid)->Option<NetConnection>{
        for c in &self.connections{
            if c.uuid == *uuid{
                return Some(c.clone())
            }
        }
        None
    }

    pub fn connection_from_address(&self, socket: &SocketAddr)->Option<NetConnection>{
        for c in &self.connections{
            if c.target == *socket{
                return Some(c.clone())
            }
        }
        None
    }

    pub fn remove_connection_for_address(&mut self, socket: &SocketAddr) -> Option<NetConnection>{
        let index = self.connections.iter().position(|c| c.target == *socket);
        index.map(|i| self.connections.swap_remove(i))
    }

}

impl<'a> Component for NetConnectionPool{
    type Storage = VecStorage<NetConnectionPool>;
}

pub struct NetSendBuffer<T> where T: Send+Sync+PartialEq+'static{
    pub buf: EventChannel<NetSourcedEvent<T>>,
}

impl<T> NetSendBuffer<T> where T: Send+Sync+PartialEq+'static{
    pub fn new()->NetSendBuffer<T>{
        NetSendBuffer{
            buf: EventChannel::<NetSourcedEvent<T>>::new(),
        }
    }
}

impl<T> Component for NetSendBuffer<T> where T: Send+Sync+PartialEq+'static{
    type Storage = VecStorage<NetSendBuffer<T>>;
}


pub struct NetReceiveBuffer<T> where T: Send+Sync+PartialEq+'static{
    pub buf: EventChannel<NetSourcedEvent<T>>,
}

impl<'a,T> Component for NetReceiveBuffer<T> where T: Send+Sync+PartialEq+'static{
    type Storage = VecStorage<NetReceiveBuffer<T>>;
}

impl<T> NetReceiveBuffer<T> where T: Send+Sync+PartialEq+'static{
    pub fn new()->NetReceiveBuffer<T>{
        NetReceiveBuffer{
            buf: EventChannel::<NetSourcedEvent<T>>::new(),
        }
    }
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