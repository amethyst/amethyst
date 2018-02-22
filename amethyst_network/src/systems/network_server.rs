//! The network server system

use specs::{System,FetchMut,Fetch};
use std::net::UdpSocket;
use std::net::IpAddr;
use std::str;
use std::net::SocketAddr;
use std::io::{Error,ErrorKind};
use shrev::*;
use std::marker::PhantomData;
use std::str::FromStr;
use std::clone::Clone;

use resources::*;
use systems::*;

use serde::Serialize;
use serde::de::DeserializeOwned;

use uuid::Uuid;

/*
TODO: State sync + Network Ownership (NetOwned)
Client Registered components: Transform Sprite LocalTransform Velocity Input Music
Server Registered components: Transform LocalTransform Velocity Input

Server->Client Event: CreateEntity with Transform(1,1,0,0)+LocalTransform([5,5,5,5],[2,2,2],[3,3,3])+NetworkedOwned(entityid:SERVERGENERATED,owner:ServerUUID)
*/

/// The System managing the server's network state and connections.
/// The T generic parameter corresponds to the network event enum type.
pub struct NetServerSystem<T>{
    /// The network socket
    pub socket: UdpSocket,
    /// The list of clients that are connected to us.
    pub clients: Vec<NetConnection>,
    net_event_types: PhantomData<T>,
}

impl<T> NetServerSystem<T> where T:Send+Sync+Serialize{
    /// Creates a NetServerSystem and binds the Socket on the ip and port added in parameters.
    pub fn new(ip:&str,port:u16)->Result<NetServerSystem<T>,Error>{
        let socket = UdpSocket::bind(SocketAddr::new(IpAddr::from_str(ip).expect("Unreadable input IP"),port))?;
        socket.set_nonblocking(true)?;
        Ok(
            NetServerSystem{
                socket,
                clients:vec![],
                net_event_types:PhantomData,
            }
        )
    }
}

//impl<T> NetworkBase<T> for NetServerSystem<T> where T:Send+Sync+Serialize+Clone+DeserializeOwned+BaseNetEvent<T>+'static{}

impl<'a, T> System<'a> for NetServerSystem<T> where T:Send+Sync+Serialize+Clone+DeserializeOwned+PartialEq+'static{
    type SystemData = (
        FetchMut<'a, NetSendBuffer<T>>,
        FetchMut<'a, NetReceiveBuffer<T>>,
    );
    //NOTE: Running it this way might cause a buffer overflow during heavy load on low-tickrate servers.
    //TODO: Once the net_debug tools will be made, test this for possible buffer overflow at OS level by monitoring packet loss in localhost.
    fn run(&mut self, (mut send_buf,mut receive_buf): Self::SystemData) {
        let mut buf = [0; 2048];
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    let net_event = deserialize_event::<T>(&buf[..amt]);
                    match net_event{
                        Ok(ev)=>{
                            let conn_index = self.clients.iter().position(|c| src == c.target);
                            match conn_index{
                                Some(ind)=>{
                                    let c = self.clients.get(ind).unwrap().clone();
                                    if c.state==ConnectionState::Connected || c.state == ConnectionState::Connecting{
                                        let owned_event = NetSourcedEvent {
                                            event:ev.clone(),
                                            connection_id:c.uuid,
                                        };
                                        receive_buf.buf.single_write(owned_event);
                                        match ev{
                                            NetEvent::Disconnect {reason}=>{
                                                self.clients.remove(ind);
                                                info!("Disconnected from server: {}",reason);
                                            }
                                            _=>{},//Other systems will handle the rest of the stuff
                                        }
                                    }else{
                                        warn!("Received message from client in invalid state connection state (not Connected and not Connecting)");
                                    }
                                },
                                None=>{
                                    //Connection protocol
                                    match ev.clone(){
                                        NetEvent::Connect=>{
                                            info!("Remote ({:?}) initialized connection sequence.",src);
                                            let conn = NetConnection{
                                                target:src,
                                                state:ConnectionState::Connecting,
                                                uuid: Uuid::new_v4(),
                                            };
                                            send_event(&NetEvent::Connected::<T>,&conn,&self.socket);

                                            //Push events to continue the user-space connection protocol
                                            let owned_event = NetSourcedEvent {
                                                event:ev,
                                                connection_id:conn.uuid,
                                            };
                                            receive_buf.buf.single_write(owned_event);

                                            self.clients.push(conn);
                                        },
                                        _=>{
                                            warn!("Received event from unknown source: {:?}",src);
                                        },
                                    }
                                },
                            }
                        },
                        Err(e)=>error!("Failed to get string from bytes: {}",e),
                    }
                },
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock{
                        break;//Safely ignores when no packets are waiting in the queue, and stop checking for this time.
                    }
                    error!("Couldn't receive a datagram: {}", e);
                },
            }
        }
    }
}
