use specs::{Entities, Entity, Join, System, WriteStorage};
use std::net::UdpSocket;
use std::str;

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

impl<'a> System<'a> for NetServerSystem {
    type SystemData = (
    );
    fn run(&mut self, (): Self::SystemData) {
        let mut buf = [0; 10];
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    println!("amt: {}", amt);
                    println!("src: {}", src);
                    println!("{}", str::from_utf8(&buf).unwrap_or(""));

                    let buf = &mut buf[..amt];
                    buf.reverse();
                    self.socket.send_to(buf, &src).expect("Failed to send data.");
                },
                Err(e) => match e {
                    WouldBlock => {break;},//Safely ignores when no packets are waiting in the queue, and stop checking for this time.
                    e => println!("couldn't receive a datagram: {}", e),
                }
            }
        }
    }
}