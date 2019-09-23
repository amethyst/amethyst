use std::net::SocketAddr;

/// Stores information about a known client for the `NetworkSimulationResource`.
#[derive(Clone, Debug)]
pub struct Client {
    addr: SocketAddr,
}

impl Client {
    /// Create and return a new Client
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    /// Return the address of the Client
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}
