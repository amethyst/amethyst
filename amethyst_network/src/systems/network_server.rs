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

use serde::{Serialize,Deserialize};

pub struct NetServerSystem<T> where T:Send+Sync{
    pub socket:UdpSocket,
    pub clients:Vec<NetConnection>,
    net_event_types:PhantomData<T>,
}

impl<T> NetServerSystem<T> where T:Send+Sync+Serialize{
    pub fn new(ip:&str,port:u16)->Result<NetServerSystem<T>,Error>{
        let mut socket = UdpSocket::bind(SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(),port))?;//TODO: Use supplied ip
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


//NOTICE ME AT REVIEW: I have no idea what I'm doing with that 'static lifetime, please tell me if its wrong.
//NOTICE ME AT REVIEW
//NOTICE ME AT REVIEW
//NOTICE ME AT REVIEW
impl<'a,T> System<'a> for NetServerSystem<T> where T:Send+Sync+Serialize+Deserialize<'a>+'static{
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
                    let conn = self.clients.iter().find(|c| src == c.target);
                    match conn{
                        Some(c)=>{
                            if c.state==ConnectionState::Connected || c.state == ConnectionState::Connecting{
                                let  buf2 = &buf[..amt];
                                let str_in = str::from_utf8(&buf2);
                                match str_in{
                                    Ok(s)=>{
                                        //TODO: Connection management server side stuff
                                        let net_event = ron::de::from_str::<T>(s);
                                        match net_event{
                                            Ok(ev)=>events.single_write(NetOwnedEvent{
                                                event:ev,
                                                owner:c.clone(),//Could be replaced by a lifetime I guess
                                        }),
                                        Err(e)=>println!("Failed to read network event!"),
                                    }
                                },
                                Err(e)=>println!("Failed to get string from bytes: {}",e),
                            }
                            }
                        },
                        None=>println!("Received network packet from unknown source."),
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