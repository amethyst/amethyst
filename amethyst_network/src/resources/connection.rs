//! Network Connection and states.

use std::net::SocketAddr;
use specs::{Component,VecStorage};
use shrev::EventChannel;
use resources::net_event::{NetEvent,NetSourcedEvent};
use uuid::Uuid;

/// A network connection target data.
//TODO add ping here?
#[derive(Debug,Clone,Serialize,Deserialize,PartialEq)]
pub struct NetConnection{
    /// The remote socket address of this connection.
    pub target: SocketAddr,
    /// The state of the connection.
    pub state: ConnectionState,
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
pub struct NetOwner{
    pub id: Uuid,
}

/// The list of network connections allocated by the network system.
pub struct NetConnectionPool{
    /// The connections.
    pub connections: Vec<NetConnection>,
}

impl<'a> Component for NetConnectionPool{
    type Storage = VecStorage<NetConnectionPool>;
}

#[derive(Debug,Clone)]
pub struct NetSendBuffer<T> where T: Send+Sync{
    pub buf: EventChannel<(Uuid,NetEvent<T>)>,
}

impl<T> Component for NetSendBuffer<T> where T: Send+Sync{
    type Storage = VecStorage<NetSendBuffer<T>>;
}

#[derive(Debug,Clone)]
pub struct NetReceiveBuffer<T> where T: Send+Sync{
    pub buf: EventChannel<NetSourcedEvent<T>>,
}

impl<'a,T> Component for NetReceiveBuffer<T> where T: Send+Sync{
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