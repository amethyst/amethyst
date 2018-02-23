extern crate bincode;

use std::net::UdpSocket;
use std::clone::Clone;
use bincode::{serialize, deserialize, Infinite};
use serde::Serialize;
use serde::de::DeserializeOwned;
use bincode::internal::ErrorKind;
use super::{NetConnection,NetConnectionPool,NetEvent};

/// Sends an event to the target NetConnection using the provided network Socket.
/// The socket has to be binded!
pub fn send_event<T>(event:&NetEvent<T>,target:&NetConnection,socket:&UdpSocket) where T:Serialize{
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
pub fn deserialize_event<T>(data:&[u8])->Result<NetEvent<T>,Box<ErrorKind>> where T:DeserializeOwned{
    deserialize::<NetEvent<T>>(data)
}

pub fn send_to<T>(event: NetEvent<T>, target: &NetConnection, pool: &mut NetConnectionPool){
    //pool.
}

/*
send_to_all
send_to_others // On server == send_to_clients old model
send_to_all_except
send_to
send_event // Internal logic
*/
