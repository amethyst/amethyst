//! The network client System

use resources::*;
use serde::Serialize;
use serde::de::DeserializeOwned;
use shrev::*;
use specs::{FetchMut, System};
use std::clone::Clone;
use std::io::{Error, ErrorKind};
use std::marker::PhantomData;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::str;
use std::str::FromStr;
use systems::NetworkBase;

/// The System managing the client's network state and connections.
/// The T generic parameter corresponds to the network event enum type.
pub struct NetClientSystem<T>
where
    T: Send + Sync,
{
    /// The network socket
    pub socket: UdpSocket,
    /// The server that we are (possibly) connected or connecting to.
    pub connection: Option<NetConnection>, //Will handle a single connection for now, as it is simple to manage and corresponds to most use cases.
    net_event_types: PhantomData<T>,
}

//TODO: add Unchecked Event type list. Those events will be let pass the client connected filter (Example: NetEvent::Connect).
//TODO: add different Filters that can be added on demand, to filter the event before they reach other systems.
impl<T> NetClientSystem<T>
where
    T: Send + Sync + Serialize + Clone + DeserializeOwned + BaseNetEvent<T> + 'static,
{
    /// Creates a NetClientSystem and binds the Socket on the ip and port added in parameters.
    pub fn new(ip: &str, port: u16) -> Result<NetClientSystem<T>, Error> {
        let socket = UdpSocket::bind(SocketAddr::new(
            IpAddr::from_str(ip).expect("Unreadable input IP"),
            port,
        ))?;
        socket.set_nonblocking(true)?;
        Ok(NetClientSystem {
            socket,
            connection: None,
            net_event_types: PhantomData,
        })
    }
    /// Connects to a remote server
    pub fn connect(&mut self, target: SocketAddr) {
        println!("Sending connection request to remote {:?}", target);
        let conn = NetConnection {
            target,
            state: ConnectionState::Connecting,
        };
        self.send_event(&T::base_to_custom(NetEvent::Connect), &conn, &self.socket);
        self.connection = Some(conn);
    }
}

impl<T> NetworkBase<T> for NetClientSystem<T>
where
    T: Send + Sync + Serialize + Clone + DeserializeOwned + BaseNetEvent<T> + 'static,
{
}

impl<'a, T> System<'a> for NetClientSystem<T>
where
    T: Send + Sync + Serialize + Clone + DeserializeOwned + BaseNetEvent<T> + 'static,
{
    type SystemData = (FetchMut<'a, EventChannel<NetOwnedEvent<T>>>,);
    //omg unreadable plz enjoy code owo
    fn run(&mut self, (mut events,): Self::SystemData) {
        let mut buf = [0; 2048];
        loop {
            match self.socket.recv_from(&mut buf) {
                //Data received
                Ok((amt, src)) => {
                    //Are we connected to anything?
                    if self.connection.is_some() {
                        //Was it sent by connected server, and are we still connected to it?
                        if src == self.connection.as_ref().unwrap().target
                            && (self.connection.as_ref().unwrap().state
                                == ConnectionState::Connected
                                || self.connection.as_ref().unwrap().state
                                    == ConnectionState::Connecting)
                        {
                            let net_event = self.deserialize_event(&buf[..amt]);
                            match net_event {
                                Ok(ev) => {
                                    let owned_event = NetOwnedEvent {
                                        event: ev.clone(),
                                        owner: self.connection.as_ref().unwrap().clone(),
                                    };
                                    events.single_write(owned_event);
                                    match T::custom_to_base(ev) {
                                        Some(NetEvent::Connected) => {
                                            self.connection.as_mut().unwrap().state =
                                                ConnectionState::Connected;
                                            println!(
                                                "Remote ({:?}) accepted connection request.",
                                                src
                                            );
                                        }
                                        //Could be handled differently by the user, say by reconnecting to a fallback server.
                                        Some(NetEvent::ConnectionRefused { reason }) => {
                                            self.connection = None;
                                            println!("Connection refused by server: {}", reason);
                                        }
                                        Some(NetEvent::Disconnected { reason }) => {
                                            self.connection = None;
                                            println!("Disconnected from server: {}", reason);
                                        }
                                        _ => {} //Other systems will handle the rest of the stuff
                                    }
                                }
                                Err(e) => println!("Failed to read network event: {}", e),
                            }
                        }
                    } else {
                        println!("Received network packet from unknown source, ignored.");
                    }
                }
                Err(e) => {
                    //No data
                    if e.kind() == ErrorKind::WouldBlock {
                        break; //Safely ignores when no packets are waiting in the queue, and stop checking for this time.,
                    }
                    println!("couldn't receive a datagram: {}", e);
                }
            }
        }
    }
}
