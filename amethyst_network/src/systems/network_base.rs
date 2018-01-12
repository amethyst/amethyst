//! The network base trait.
//! Used to share code between the client and the server.

extern crate bincode;

use bincode::{deserialize, serialize, Infinite};
use bincode::internal::ErrorKind;
use resources::*;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::clone::Clone;
use std::net::UdpSocket;

/// The NetworkBase trait is used to share the code between the client and the server network systems.
/// The T generic parameter corresponds to the network event enum type.
pub trait NetworkBase<T>
where
    T: Send + Sync + Serialize + Clone + DeserializeOwned + BaseNetEvent<T> + 'static,
{
    /// Sends an event to the target NetConnection using the provided network Socket.
    /// The socket has to be binded!
    fn send_event(&self, event: &T, target: &NetConnection, socket: &UdpSocket) {
        let ser = serialize(event, Infinite);
        match ser {
            Ok(s) => {
                if let Err(e) = socket.send_to(s.as_slice(), target.target) {
                    println!("Failed to send data to network socket: {}", e);
                }
            }
            Err(e) => println!("Failed to serialize the event: {}", e),
        }
    }

    /// Attempts to deserialize an event from the raw byte data.
    fn deserialize_event(&self, data: &[u8]) -> Result<T, Box<ErrorKind>> {
        deserialize::<T>(data)
    }
}
