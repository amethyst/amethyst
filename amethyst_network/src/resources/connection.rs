//! Network Connection and states

use std::net::SocketAddr;

///Connection target
//TODO add ping
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NetConnection{
    /// The remote socket address of this connection
    pub target:SocketAddr,
    /// The state of the connection
    pub state:ConnectionState,
}

impl PartialEq for NetConnection {
    fn eq(&self, other: &NetConnection) -> bool {
        self.target == other.target
    }
}

impl Eq for NetConnection{}

///The state of the connection
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub enum ConnectionState{
    /// The connection is established
    Connected,
    /// The connection is being established
    Connecting,
    /// The connection has been dropped
    Disconnected,
}