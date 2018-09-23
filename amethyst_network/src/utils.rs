extern crate bincode;

use super::NetEvent;
use bincode::internal::ErrorKind;
use bincode::{deserialize, serialize, Infinite};

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::net::SocketAddr;
use std::net::UdpSocket;

/// Sends an event to the target NetConnection using the provided network Socket.
/// The socket has to be bound.
pub fn send_event<T>(event: &NetEvent<T>, target: &SocketAddr, socket: &UdpSocket)
where
    T: Serialize,
{
    let ser = serialize(event, Infinite);
    match ser {
        Ok(s) => {
            let slice = s.as_slice();
            match socket.send_to(slice, target) {
                Ok(_qty) => {}
                Err(e) => error!("Failed to send data to network socket: {}", e),
            }
        }
        Err(e) => error!("Failed to serialize the event: {}", e),
    }
}

/// Attempts to deserialize an event from the raw byte data.
pub fn deserialize_event<T>(data: &[u8]) -> Result<NetEvent<T>, Box<ErrorKind>>
where
    T: DeserializeOwned,
{
    deserialize::<NetEvent<T>>(data)
}
