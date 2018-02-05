//! Network Connection and states.

use std::net::SocketAddr;
use specs::{Component,VecStorage};
use shrev::EventChannel;
use resources::net_event::{NetEvent,NetSourcedEvent};

///Connection target.
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

/// The list of network connections allocated by the network system.
pub struct NetConnectionPool<'a>{
    /// The connections.
    pub connections: Vec<&'a NetConnection>,
}

impl<'a> Component for NetConnectionPool<'a>{
    type Storage = VecStorage<NetConnectionPool<'a>>;
}

pub enum NetBuffer<'a,T>{
    NetSendBuffer(EventChannel<(&'a NetConnection,&'a NetEvent<T>)>),
    NetReceiveBuffer(EventChannel<&'a NetSourcedEvent<'a,T>>),
}

impl<'a,T> Component for NetBuffer<'a,T>{
    type Storage = VecStorage<NetBuffer<'a,T>>;
}

/*pub struct NetSendBuffer<'a,T>{
    pub buf: EventChannel<(&'a NetConnection,&'a NetEvent<T>)>,
}*/