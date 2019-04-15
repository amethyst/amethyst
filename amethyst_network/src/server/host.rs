//! This module is about the communication with this host to some other endpoints.
//!
//! 1. Sending Data
//! 2. Receiving Data
//! 3. Broadcasting

use crate::{error::Result, server::ServerConfig};
use crossbeam_channel::{Receiver, Sender};
use laminar::{Packet, Socket, SocketEvent};
use std::thread;

/// 'Host' abstracts Laminar udp sockets away.
pub struct Host {
    packet_sender: Sender<Packet>,
    packet_receiver: Receiver<SocketEvent>,
}

impl Host {
    /// This will start and return an instance of the host.
    ///
    /// The method uses the config provided when creating a `host` instance.
    pub fn run(config: &ServerConfig) -> Result<Host> {
        let (mut socket, packet_sender, packet_receiver) = Socket::bind(config.udp_socket_addr)?;

        thread::spawn(move || {
            socket.start_polling().unwrap();
        });

        Ok(Host {
            packet_sender,
            packet_receiver,
        })
    }

    /// Get the handle to the internals of the UDP-receiving threat.
    pub fn udp_receive_handle(&self) -> Receiver<SocketEvent> {
        self.packet_receiver.clone()
    }

    /// Get the handle to the internals of the UDP-sending thread.
    pub fn udp_send_handle(&self) -> Sender<Packet> {
        self.packet_sender.clone()
    }

    /// Schedule a UDP-packet for sending.
    pub fn send_udp(&mut self, packet: Packet) -> Result<()> {
        self.packet_sender.send(packet)?;
        Ok(())
    }
}
