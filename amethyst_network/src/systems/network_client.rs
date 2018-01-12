extern crate ron;
extern crate serde_json;

use specs::{Entities, Entity, Join, System, WriteStorage,Component,VecStorage,FetchMut};
use std::net::UdpSocket;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::io::{Error,ErrorKind};
use std::str;
use std::str::FromStr;
use std::clone::Clone;


use amethyst_core::transform::*;

use shrev::*;

use components::netsync::*;
use resources::net_event::*;
use network_server::*;
use resources::connection::*;
use std::marker::PhantomData;

use serde::Serialize;
use serde::de::DeserializeOwned;

pub struct NetClientSystem<T> where T:Send+Sync{
    pub socket:UdpSocket,
    pub connection:Option<NetConnection>,//Will handle a single connection for now, as it is simple to manage.
    net_event_types:PhantomData<T>,
}

impl<T> NetClientSystem<T> where T:Send+Sync+Serialize+BaseNetEvent<T>{
    pub fn new(ip:&str,port:u16)->Result<NetClientSystem<T>,Error>{
        let mut socket = UdpSocket::bind(SocketAddr::new(IpAddr::from_str(ip).expect("Unreadable input IP"),port))?;//TODO: Use supplied ip
        socket.set_nonblocking(true);
        Ok(
            NetClientSystem{
                socket,
                connection:None,
                net_event_types:PhantomData,
            }
        )
    }

    pub fn connect(&mut self,target:SocketAddr){
        println!("Sending connection request to remote {:?}",target);
        self.connection = Some(
            NetConnection{
                target,
                state:ConnectionState::Connecting,
            }
        );
        self.send_event(T::base_to_custom(NetEvent::Connect));//FIXME: I think this should use associated data/constant/type to work.  ConnectionEventType=NetEvent::Connect
    }

    pub fn send_event(&mut self,event:T){
        //Possible to have a better syntax? :/
        match self.connection{
            Some(ref mut conn)=>{
                let ser = ron::ser::pretty::to_string(&event);
                //let s = serde_json::ser::;
                match ser{
                    Ok(s)=>{
                        let mut buf = s.as_bytes();//temporary, so we know what we are doing. Will be replaced by serde_json::ser::to_bytes
                        let res = self.socket.send_to(buf, conn.target);
                    },
                    Err(e)=>println!("Failed to serialize the event: {}",e),
                }
            },
            None=>println!("Cannot send an event, as we are not connected!"),
        }
    }
}

impl<'a, T> System<'a> for NetClientSystem<T> where T:Send+Sync+Serialize+Clone+DeserializeOwned+BaseNetEvent<T>+'static{
    type SystemData = (
        FetchMut<'a, EventChannel<NetOwnedEvent<T>>>,
    );
    //omg unreadable plz enjoy code owo
    fn run(&mut self, (mut events,): Self::SystemData) {
        let mut buf = [0; 2048];
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((amt, src)) => { //Data received
                    if self.connection.is_some(){ //Are we connected to anything?
                        if src == self.connection.as_ref().unwrap().target && (self.connection.as_ref().unwrap().state == ConnectionState::Connected || self.connection.as_ref().unwrap().state == ConnectionState::Connecting){ //Was it sent by connected server, and are we still connected to it?
                            let mut buf2:&[u8] = &buf[..amt];
                            let str_in = str::from_utf8(&buf2);
                            match str_in{
                                Ok(s)=>{
                                    let net_event = ron::de::from_str::<T>(s);
                                    match net_event{
                                        Ok(ev)=>{
                                            let owned_event = NetOwnedEvent{
                                                event:ev.clone(),
                                                owner:self.connection.as_ref().unwrap().clone(),
                                            };
                                            events.single_write(owned_event);
                                            match T::custom_to_base(ev){
                                                Some(NetEvent::Connected)=>{
                                                    self.connection.as_mut().unwrap().state = ConnectionState::Connected;
                                                    println!("Remote ({:?}) accepted connection request.",src);
                                                },
                                                Some(NetEvent::ConnectionRefused {reason})=>{ //Could be handled differently by the user, say by reconnecting to a fallback server.
                                                    self.connection = None;
                                                    println!("Connection refused by server: {}",reason);
                                                },
                                                Some(NetEvent::Disconnected {reason})=>{
                                                    self.connection = None;
                                                    println!("Disconnected from server: {}",reason);
                                                }
                                                _=>{},//Other systems will handle the rest of the stuff
                                            }
                                        },
                                        Err(e)=>println!("Failed to read network event!"),
                                    }
                                },
                                Err(e)=>println!("Failed to get string from bytes: {}",e),
                            }
                        }
                    }
                    else{
                        println!("Received network packet from unknown source, ignored.");
                    }
                },
                Err(e) => { //No data
                    if e.kind() == ErrorKind::WouldBlock{
                        break;//Safely ignores when no packets are waiting in the queue, and stop checking for this time.,
                    }
                    println!("couldn't receive a datagram: {}", e);
                },
            }
        }
    }
}
