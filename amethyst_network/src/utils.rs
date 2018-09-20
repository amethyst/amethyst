extern crate bincode;

use super::{NetConnection, NetEvent};
use bincode::internal::ErrorKind;
use bincode::{deserialize, serialize, Infinite};
//use mio::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::clone::Clone;
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
                Ok(qty) => {}
                Err(e) => error!("Failed to send data to network socket: {}", e),
            }
            //sleep(Duration::from_millis(1));
            /*if let Err(e) = socket.send_to(slice, target) {
                error!("Failed to send data to network socket: {}", e);
            }*/        }
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
/*
/// Send an event to a network connection by adding the event to the send queue.
/// This will eventually have support for reliability settings.
pub fn send_to<T>(event: NetEvent<T>, buf: &mut NetSendBuffer<T>, target: &NetConnection)
where
    T: Send + Sync + 'static,
{
    buf.buf.single_write(NetSourcedEvent {
        event,
        uuid: target.uuid,
        socket: target.target,
    });
}

/// Sends an event to all connections in the NetConnectionPool.
pub fn send_to_all<T>(event: NetEvent<T>, buf: &mut NetSendBuffer<T>, pool: &NetConnectionPool)
where
    T: Send + Sync + Clone + 'static,
{
    for conn in &pool.connections {
        send_to(event.clone(), buf, conn);
    }
}

/// Sends an event to all connections in the NetConnectionPool ignoring the specified network connection.
pub fn send_to_all_except<T>(
    event: NetEvent<T>,
    buf: &mut NetSendBuffer<T>,
    pool: &NetConnectionPool,
    except: &NetConnection,
) where
    T: Send + Sync + Clone + 'static,
{
    for conn in pool.connections.iter().filter(|&&ref c| c != except) {
        send_to(event.clone(), buf, conn);
    }
}
*/
