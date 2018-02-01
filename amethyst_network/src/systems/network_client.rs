//! The network client System

use specs::{System,FetchMut};
use std::net::UdpSocket;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::io::{Error,ErrorKind};
use std::str;
use std::str::FromStr;
use std::clone::Clone;

use shrev::*;

use resources::*;
use systems::*;
use std::marker::PhantomData;

use serde::Serialize;
use serde::de::DeserializeOwned;

/// The System managing the client's network state and connections.
/// The T generic parameter corresponds to the network event enum type.
pub struct NetClientSystem<T>{
    /// The network socket
    pub socket: UdpSocket,
    /// The server that we are (possibly) connected or connecting to.
    pub connection: Option<NetConnection>,//Will handle a single connection for now, as it is simple to manage and corresponds to most use cases.
    net_event_types: PhantomData<T>,
}

//TODO: add Unchecked Event type list. Those events will be let pass the client connected filter (Example: NetEvent::Connect).
//TODO: add different Filters that can be added on demand, to filter the event before they reach other systems.
impl<T> NetClientSystem<T> where T:Send+Sync+Serialize+Clone+DeserializeOwned+'static{
    /// Creates a NetClientSystem and binds the Socket on the ip and port added in parameters.
    pub fn new(ip:&str,port:u16)->Result<NetClientSystem<T>,Error>{
        let socket = UdpSocket::bind(SocketAddr::new(IpAddr::from_str(ip).expect("Unreadable input IP"),port))?;
        socket.set_nonblocking(true)?;
        Ok(
            NetClientSystem{
                socket,
                connection:None,
                net_event_types:PhantomData,
            }
        )
    }
    /// Connects to a remote server
    pub fn connect(&mut self,target:SocketAddr){
        println!("Sending connection request to remote {:?}",target);
        let conn = NetConnection{
            target,
            state:ConnectionState::Connecting,
        };
        send_event(NetEvent::<T>::Connect,&conn,&self.socket);
        self.connection = Some(
            conn,
        );
    }
}

//impl<T> NetworkBase<T> for NetClientSystem<T> where T:Send+Sync+Serialize+Clone+DeserializeOwned+BaseNetEvent<T>+'static{}

impl<'a, T> System<'a> for NetClientSystem<T> where T:Send+Sync+Serialize+Clone+DeserializeOwned+'static{
    type SystemData = FetchMut<'a, EventChannel<NetOwnedEvent<NetEvent<T>>>>;
    fn run(&mut self, mut events: Self::SystemData) {
        let mut buf = [0; 2048];
        loop {
            match self.socket.recv_from(&mut buf) {
                //Data received
                Ok((amt, src)) => {
                    //Are we connected to anything?
                    let mut connection_dropped = false;
                    if let Some(mut conn) = self.connection.as_mut(){
                        //Was it sent by connected server, and are we still connected to it?
                        if src == conn.target && (conn.state == ConnectionState::Connected || conn.state == ConnectionState::Connecting){
                            let net_event = deserialize_event::<T>(&buf[..amt]);
                            match net_event{
                                Ok(ev)=>{
                                    let owned_event = NetOwnedEvent{
                                        event:ev.clone(),
                                        owner:conn.clone(),
                                    };
                                    events.single_write(owned_event);
                                    match ev{
                                        NetEvent::Connected=>{
                                            conn.state = ConnectionState::Connected;
                                            info!("Remote ({:?}) accepted connection request.",src);
                                        },
                                        //Could be handled differently by the user, say by reconnecting to a fallback server.
                                        NetEvent::ConnectionRefused {reason}=>{
                                            connection_dropped = true;
                                            info!("Connection refused by server: {}",reason);
                                        },
                                        NetEvent::Disconnected {reason}=>{
                                            connection_dropped = true;
                                            info!("Disconnected from server: {}",reason);
                                        }
                                        _=>{},//Other systems will handle the rest of the stuff
                                    }
                                },
                                Err(e)=>error!("Failed to read network event: {}",e),
                            }
                        }
                    }
                    else{
                        warn!("Received network packet from unknown source, ignored.");
                    }
                    if connection_dropped{
                        self.connection = None;
                    }
                },
                Err(e) => { //No data
                    if e.kind() == ErrorKind::WouldBlock{
                        break;//Safely ignores when no packets are waiting in the queue, and stop checking for this time.,
                    }
                    error!("couldn't receive a datagram: {}", e);
                },
            }
        }
    }
}
