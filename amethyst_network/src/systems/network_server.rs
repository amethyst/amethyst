extern crate ron;

use specs::{Entities, Entity, Join, System, WriteStorage};
use std::net::UdpSocket;
use std::str;
use std::io::{Error,ErrorKind};
use amethyst_core::transform::*;

pub struct NetServerSystem {
    pub socket:UdpSocket,
}

impl NetServerSystem {
    pub fn new()->NetServerSystem{
        println!("Starting server socket");
        let mut socket = UdpSocket::bind("127.0.0.1:34255").expect("Failed to bind to port.");
        socket.set_nonblocking(true);

        NetServerSystem{
            socket
        }
    }
}


//Event, State
//Event::CreateState(state)
//Event::UpdateState(stateid,newstate)
//Event::DeleteState(stateid)


//NetEvent(str)
//NetEvent("delete")


//Events{1=>Move}




/*


Client Registered components: Transform Sprite LocalTransform Velocity Input Music
Server Registered components: Transform LocalTransform Velocity Input

Server->Client Event: Create Entity with Transform(1,1,0,0)+LocalTransform([5,5,5,5],[2,2,2],[3,3,3])+NetworkedOwned(entityid:SERVERGENERATED,owner:ServerUUID)




*/


impl<'a> System<'a> for NetServerSystem {
    type SystemData = (
    );
    fn run(&mut self, (): Self::SystemData) {
        let mut buf = [0; 2048];
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    println!("amt: {}", amt);
                    println!("src: {}", src);
                    println!("{}", str::from_utf8(&buf).unwrap_or(""));

                    let  buf2 = &buf[..amt];

                    let strin = str::from_utf8(&buf2).expect("Failed to get string from bytes");

                    let tr:Transform = ron::de::from_str(strin).expect("Failed to get transform from string");

                    println!("{:?}",tr);

                    /*let buf = &mut buf[..amt];
                    buf.reverse();
                    self.socket.send_to(buf, &src).expect("Failed to send data.");*/
                },
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock{
                        break;//Safely ignores when no packets are waiting in the queue, and stop checking for this time.,
                    }
                    println!("couldn't receive a datagram: {}", e);
                },
            }
        }
    }
}