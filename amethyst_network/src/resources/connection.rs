use std::net::SocketAddr;

///Connection target
//TODO add ping
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NetConnection{
    pub target:SocketAddr,
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
    Connected,
    Connecting,
    Disconnected,
}