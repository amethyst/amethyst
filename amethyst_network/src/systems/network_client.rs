extern crate ron;
extern crate serde_json;

use specs::{Entities, Entity, Join, System, WriteStorage};
use std::net::UdpSocket;
use std::net::IpAddr;
use std::str;

use amethyst_core::transform::*;

use ::components::netsync::*;
use ::network_server::*;

pub struct NetClientSystem {
    pub socket:UdpSocket,
}

impl NetClientSystem {
    pub fn new()->NetClientSystem{
        println!("Starting client socket");
        let mut socket = UdpSocket::bind("127.0.0.1:34254").expect("Failed to bind to port.");

        let srv = "127.0.0.1:34255";

        //let mut buf = [0; 10];
        /*let mut buf = "yeet xD".as_bytes().clone();
        socket.send_to(buf, &srv).expect("Failed to send data.");

        let mut buf = [0;50];
        let (amt, src) = socket.recv_from(&mut buf).expect("Failed to receive data.");
        println!("{}",str::from_utf8(&buf).expect("Failed to read string from bytes"));*/


        //let ser = Transform::default();
        //let ser = LocalTransform::default();
        //let ser = TestEvent::C{c1:5,c2:"this is a test string, hello everyone".to_string()};//64 bytes received
        let ser = TestEvent::B;//1 byte received
        let s = ron::ser::pretty::to_string(&ser).unwrap();
        //let s = ron::ser::to_bytes(&ser).unwrap();
        //let s = serde_json::ser::;
        println!("{}",s);
        let mut buf = s.as_bytes();
        socket.send_to(buf, &srv).expect("Failed to send data.");

        NetClientSystem{
            socket
        }
    }
}

impl<'a> System<'a> for NetClientSystem {
    type SystemData = (
    );
    fn run(&mut self, (): Self::SystemData) {

    }
}