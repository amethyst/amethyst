use std::net::SocketAddr;

#[derive(Clone, Debug)]
/// The configuration used for the networking system.
pub struct ServerConfig {
    /// Address at which the UDP server will listen for incoming packets.
    pub udp_recv_addr: SocketAddr,
    /// Address from which the UDP server will be sending packets.
    pub udp_send_addr: SocketAddr,
    /// Specifies what the maximal packets that could be handled by the server.
    /// This value is meant for preventing some loops to read infinitely long when many packets are send and received.
    /// This value is by default 5000.
    pub max_throughput: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            // by passing in :0 port the OS will give an available port.
            udp_recv_addr: "0.0.0.0:0".parse().unwrap(),
            udp_send_addr: "0.0.0.0:0".parse().unwrap(),
            max_throughput: 5000,
        }
    }
}
