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
pub(crate) fn send_event<T>(event:&NetEvent<T>,target:&NetConnection,socket:&UdpSocket) where T:Serialize+PartialEq{
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
pub(crate) fn deserialize_event<T>(data:&[u8])->Result<NetEvent<T>,Box<ErrorKind>> where T:DeserializeOwned+PartialEq{
    deserialize::<NetEvent<T>>(data)
}

pub fn send_to<T>(event: NetEvent<T>, target: &NetConnection, pool: &mut NetConnectionPool) where T: PartialEq{
    //pool.
}

/*
send_to_all
send_to_others // On server == send_to_clients old model
send_to_all_except
send_to
send_event // Internal logic
*/
