//! The network base trait.
//! Used to share code between the client and the server.

extern crate bincode;

use std::net::UdpSocket;
use std::clone::Clone;
use bincode::{serialize, deserialize, Infinite};
use resources::*;

use serde::Serialize;
use serde::de::DeserializeOwned;
use bincode::internal::ErrorKind;

/*/// The NetworkBase trait is used to share the code between the client and the server network systems.
/// The T generic parameter corresponds to the network event enum type.
pub trait NetworkBase<T> where T:Send+Sync+Serialize+Clone+DeserializeOwned+'static{
}*/

/// Sends an event to the target NetConnection using the provided network Socket.
/// The socket has to be binded!
pub fn send_event<T>(event:&NetEvent<T>,target:&NetConnection,socket:&UdpSocket) where T:Send+Sync+Serialize+Clone+DeserializeOwned+'static{
    let ser = serialize(event, Infinite);
    match ser{
        Ok(s)=>{
            let slice = s.as_slice();
            if let Err(e) = socket.send_to(slice, target.target){
                error!("Failed to send data to network socket: {}",e);
            }
        },
        Err(e)=>error!("Failed to serialize the event: {}",e),
    }
}

/// Attempts to deserialize an event from the raw byte data.
pub fn deserialize_event<T>(data:&[u8])->Result<T,Box<ErrorKind>> where T:Send+Sync+Serialize+Clone+DeserializeOwned+'static{
    deserialize::<NetEvent<T>>(data)
}