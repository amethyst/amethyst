use bytes::Bytes;
use std::net::SocketAddr;

/// Events which can be received from the network.
#[derive(Debug)]
pub enum NetworkSimulationEvent {
    // A message was received from a remote client
    Message(SocketAddr, Bytes),
    // A new host has connected to us
    Connect(SocketAddr),
    // A host has disconnected from us
    Disconnect(SocketAddr),
}
