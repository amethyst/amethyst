//! The network client System

use specs::{System,FetchMut,Fetch};
use specs::saveload::U64Marker;
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

use uuid::Uuid;

/// The System managing the client's network state and connections.
/// The T generic parameter corresponds to the network event enum type.
pub struct NetClientSystem<T>{
    /// The network socket
    pub socket: UdpSocket,
    pub send_queue_reader: ReaderId<U64Marker>,
    /// The server that we are (possibly) connected or connecting to.
    //pub connection: Option<NetConnection>,//Will handle a single connection for now, as it is simple to manage and corresponds to most use cases.
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
                send_queue_reader: None,
                net_event_types:PhantomData,
            }
        )
    }
    /// Connects to a remote server
    pub fn connect(&mut self,target:SocketAddr,pool: &mut NetConnectionPool){
        println!("Sending connection request to remote {:?}",target);
        let conn = NetConnection{
            target,
            state:ConnectionState::Connecting,
            uuid: Uuid::new_v4(),
        };
        send_event(&NetEvent::Connect::<T>,&conn,&self.socket);
        pool.connections.push(conn);
    }
}

//impl<T> NetworkBase<T> for NetClientSystem<T> where T:Send+Sync+Serialize+Clone+DeserializeOwned+BaseNetEvent<T>+'static{}

impl<'a, T> System<'a> for NetClientSystem<T> where T:Send+Sync+Serialize+Clone+DeserializeOwned+'static{
    type SystemData = (
        Fetch<'a, NetSendBuffer<T>>,
        FetchMut<'a, NetReceiveBuffer<T>>,
        Fetch<'a, NetConnectionPool>,
    );
    fn run(&mut self, (send_buf,mut receive_buf,pool): Self::SystemData) {
        //Tx
        if self.send_queue_reader.is_none(){
            self.send_queue_reader = Some(send_buf.buf.register_reader());
        }

        for ev in send_buf.buf.read(self.send_queue_reader.as_mut().unwrap()){
            info!("NET EVENT TO SEND");
            let target = pool.connection_from_uuid(ev.connection_id);
            if let Some(t) = target{
                if t.state == ConnectionState::Connected || t.state == ConnectionState::Connecting{
                    send_event(&ev.event,t,&self.socket);
                }
            }
        }

        // Rx
        let mut buf = [0; 2048];
        loop {
            match self.socket.recv_from(&mut buf) {
                //Data received
                Ok((amt, src)) => {
                    //Are we connected to anything?
                    let mut connection_dropped = false;
                    if let Some(mut conn) = pool.connections.first().as_mut(){
                        //Was it sent by connected server, and are we still connected to it?
                        if src == conn.target && (conn.state == ConnectionState::Connected || conn.state == ConnectionState::Connecting){
                            let net_event = deserialize_event::<T>(&buf[..amt]);
                            match net_event{
                                Ok(ev)=>{
                                    let owned_event = NetSourcedEvent {
                                        event:ev.clone(),
                                        connection_id:conn.uuid,
                                    };
                                    receive_buf.buf.single_write(owned_event);
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
                        pool.connections.pop();
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
