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

use serde::{Serialize,Deserialize};

pub struct NetClientSystem<T> where T:Send+Sync{
    pub socket:UdpSocket,
    pub connection:Option<NetConnection>,//Will handle a single connection for now, as it is simple to manage.
    net_event_types:PhantomData<T>,
}

impl<T> NetClientSystem<T> where T:Send+Sync+Serialize{
    pub fn new(ip:&str,port:u16)->Result<NetClientSystem<T>,Error>{
        let mut socket = UdpSocket::bind(SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(),port))?;//TODO: Use supplied ip
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
        self.connection = Some(
            NetConnection{
                target,
                state:ConnectionState::Connecting,
            }
        );
        self.send_event(NetEvent::Connect);//FIXME: I think this should use associated data/constant/type to work.  ConnectionEventType=NetEvent::Connect
    }

    pub fn send_event(&mut self,event:T){
        //Possible to have a better syntax? :/
        match self.connection{
            Some(conn)=>{
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


//NOTICE ME AT REVIEW: I have no idea what I'm doing with that 'static lifetime, please tell me if its wrong.
//NOTICE ME AT REVIEW
//NOTICE ME AT REVIEW
//NOTICE ME AT REVIEW
impl<'a,T> System<'a> for NetClientSystem<T> where T:Send+Sync+Serialize+Deserialize<'a>+'static{
    type SystemData = (
        FetchMut<'a, EventChannel<NetOwnedEvent<T>>>,
    );
    //omg unreadable plz enjoy code owo
    fn run(&mut self, (mut events,): Self::SystemData) {
        let mut buf = [0; 2048];
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((amt, src)) => { //Data received
                    match self.connection{
                        Some(conn)=>{ //Are we connected to anything?
                            if src == conn.target && (conn.state == ConnectionState::Connected || conn.state == ConnectionState::Connecting){ //Was it sent by connected server, and are we still connected to it?
                                let buf2 = &buf[..amt];
                                let str_in = str::from_utf8(&buf2);
                                match str_in{
                                    Ok(s)=>{
                                        let net_event = ron::de::from_str::<T>(s);
                                        match net_event{
                                            Ok(ev)=>{
                                                let owned_event = NetOwnedEvent{
                                                    event:ev,
                                                    owner:conn.clone(),
                                                };
                                                events.single_write(owned_event);
                                                match ev{
                                                    NetEvent::Connected=>conn.state = ConnectionState::Connected,
                                                    NetEvent::ConnectionRefused {reason}=>{ //Could be handled differently by the user, say by reconnecting to a fallback server.
                                                        self.connection = None;
                                                        println!("Connection refused by server: {}",reason);
                                                    },
                                                    NetEvent::Disconnected {reason}=>{
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
                        },
                        None=>println!("Received network packet from unknown source, ignored."),
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