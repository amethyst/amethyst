use specs::{Entities, Entity, Join, System, WriteStorage};
use std::net::UdpSocket;
use std::net::IpAddr;
use std::str;

pub struct NetClientSystem {
    pub socket:UdpSocket,
}

impl NetClientSystem {
    pub fn new()->NetClientSystem{
        println!("Starting client socket");
        let mut socket = UdpSocket::bind("127.0.0.1:34254").expect("Failed to bind to port.");

        let srv = "127.0.0.1:34255";

        //let mut buf = [0; 10];
        let mut buf = "yeet xD".as_bytes().clone();
        socket.send_to(buf, &srv).expect("Failed to send data.");

        let mut buf = [0;50];
        let (amt, src) = socket.recv_from(&mut buf).expect("Failed to receive data.");
        println!("{}",str::from_utf8(&buf).expect("Failed to read string from bytes"));

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