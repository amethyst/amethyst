extern crate ron;

use specs::{Entities, Entity, Join, System, WriteStorage,Component,VecStorage,FetchMut};
use std::net::UdpSocket;
use std::net::IpAddr;
use std::str;
use std::net::SocketAddr;
use std::io::{Error,ErrorKind};
use amethyst_core::transform::*;
use std::any::Any;
use shrev::*;
use std::marker::PhantomData;
use std::str::FromStr;
use std::clone::Clone;

use resources::connection::*;
use resources::net_event::*;

use serde::Serialize;
use serde::de::DeserializeOwned;

pub struct NetServerSystem<T> where T:Send+Sync{
    pub socket:UdpSocket,
    pub clients:Vec<NetConnection>,
    net_event_types:PhantomData<T>,
}

impl<T> NetServerSystem<T> where T:Send+Sync+Serialize{
    pub fn new(ip:&str,port:u16)->Result<NetServerSystem<T>,Error>{
        let mut socket = UdpSocket::bind(SocketAddr::new(IpAddr::from_str(ip).expect("Unreadable input IP"),port))?;
        socket.set_nonblocking(true);
        Ok(
            NetServerSystem{
                socket,
                clients:vec![],
                net_event_types:PhantomData,
            }
        )
    }

    pub fn send_event(&mut self,event:T,target:NetConnection){
        let ser = ron::ser::pretty::to_string(&event);
        //let s = serde_json::ser::;
        match ser{
            Ok(s)=>{
                let mut buf = s.as_bytes();//temporary, so we know what we are doing. Will be replaced by serde_json::ser::to_bytes
                let res = self.socket.send_to(buf, target.target);
            },
            Err(e)=>println!("Failed to serialize the event: {}",e),
        }
    }
}
/*
Client Registered components: Transform Sprite LocalTransform Velocity Input Music
Server Registered components: Transform LocalTransform Velocity Input

Server->Client Event: Create Entity with Transform(1,1,0,0)+LocalTransform([5,5,5,5],[2,2,2],[3,3,3])+NetworkedOwned(entityid:SERVERGENERATED,owner:ServerUUID)
*/

impl<'a, T> System<'a> for NetServerSystem<T> where T:Send+Sync+Serialize+Clone+DeserializeOwned+BaseNetEvent<T>+'static{
    type SystemData = (
        FetchMut<'a, EventChannel<NetOwnedEvent<T>>>,
    );
    //NOTE: Running it this way might cause a buffer overflow during heavy load on low-tickrate servers.
    //TODO: Once the net_debug tools will be made, test this for possible buffer overflow at OS level by monitoring packet loss in localhost.
    fn run(&mut self, (mut events,): Self::SystemData) {
        let mut buf = [0; 2048];
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    let  buf2 = &buf[..amt];
                    let str_in = str::from_utf8(&buf2);
                    match str_in{
                        Ok(s)=>{
                            let net_event = ron::de::from_str::<T>(s);
                            match net_event{
                                Ok(ev)=>{
                                    let conn_index = self.clients.iter().position(|c| src == c.target);
                                    match conn_index{
                                        Some(ind)=>{
                                            let c = self.clients.get(ind).unwrap().clone();
                                            if c.state==ConnectionState::Connected || c.state == ConnectionState::Connecting{
                                                let owned_event = NetOwnedEvent{
                                                    event:ev.clone(),
                                                    owner:c.clone(),
                                                };
                                                events.single_write(owned_event);
                                                match T::custom_to_base(ev){
                                                    Some(NetEvent::Disconnect {reason})=>{
                                                        self.clients.remove(ind);
                                                        println!("Disconnected from server: {}",reason);
                                                    }
                                                    _=>{},//Other systems will handle the rest of the stuff
                                                }
                                            }else{
                                                println!("Received message from client in invalid state connection state (not Connected and not Connecting)");
                                            }
                                        },
                                        None=>{
                                            //Connection protocol
                                            match T::custom_to_base(ev.clone()){
                                                Some(NetEvent::Connect)=>{
                                                    println!("Remote ({:?}) initialized connection sequence.",src);
                                                    let conn = NetConnection{
                                                        target:src,
                                                        state:ConnectionState::Connecting,
                                                    };
                                                    self.send_event(T::base_to_custom(NetEvent::Connected),conn.clone());

                                                    //Push events to continue the user-space connection protocol
                                                    let owned_event = NetOwnedEvent{
                                                        event:ev,
                                                        owner:conn.clone(),
                                                    };
                                                    events.single_write(owned_event);

                                                    self.clients.push(conn);
                                                },
                                                _=>{
                                                    println!("Received event from unknown source: {:?}",src);
                                                },
                                            }
                                        },
                                    }
                                },
                                Err(e)=>println!("Failed to read network event!"),
                            }
                        },
                        Err(e)=>println!("Failed to get string from bytes: {}",e),
                    }
                },
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock{
                        break;//Safely ignores when no packets are waiting in the queue, and stop checking for this time.,
                    }
                    println!("Couldn't receive a datagram: {}", e);
                },
            }
        }
    }
}
