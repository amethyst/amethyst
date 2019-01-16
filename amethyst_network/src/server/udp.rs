//! All UDP related logic for getting and sending data out to the other side.

use crate::{
    error::Result,
    server::{
        ClientEvent, PacketReceiving, PacketSending, ReceiveHandler, SendHandler, ServerConfig,
        ServerSocketEvent,
    },
};
use laminar::{net::UdpSocket, NetworkConfig};
use log::{error, warn};
use std::{
    net::SocketAddr,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

/// An UDP receiver, wrapper for starting the UDP-receiving thread.
pub struct UdpReceiver {
    // socket used for receiving packets
    socket: UdpSocket,
    /// The max throughput in packets a host can read at once
    pub max_throughput: usize,
}

impl UdpReceiver {
    /// This will run the udp receiver on it's own thread.
    pub fn run(addr: SocketAddr, config: &ServerConfig) -> Result<ReceiveHandler> {
        let socket = UdpSocket::bind(&addr, NetworkConfig::default())?;

        let mut receiver = UdpReceiver {
            socket,
            max_throughput: config.max_throughput as usize,
        };

        // channel used for communicating about received packets.
        let (tx, rx) = mpsc::channel();

        let thread_handle = thread::spawn(move || {
            receiver.start_receiving(tx);
        });

        Ok(ReceiveHandler::new(rx, thread_handle))
    }
}

impl PacketReceiving for UdpReceiver {
    // 1. Receive data from socket.
    // 2. Analyze data.
    // 3. Check if there are events from laminar.
    // 3. Notify the receiver with the received data over send channel.
    fn start_receiving(&mut self, tx: Sender<ServerSocketEvent>) {
        loop {
            let result = self.socket.recv();
            match result {
                Ok(Some(packet)) => {
                    if let Err(e) = tx.send(ServerSocketEvent::Packet(packet)) {
                        error!("Send channel error. Reason: {:?}", e)
                    }
                }
                Ok(None) => {
                    if let Err(e) = tx.send(ServerSocketEvent::Empty) {
                        error!("Send channel error. Reason: {:?}", e)
                    }
                }
                Err(e) => {
                    if let Err(e) = tx.send(ServerSocketEvent::Error(e)) {
                        error!("Send channel error. Reason: {:?}", e)
                    }
                }
            };

            // iter over client events occurred from within laminar.
            self.socket.events().into_iter().for_each(|event| {
                if let Err(e) = tx.send(ServerSocketEvent::ClientEvent(ClientEvent::from(event))) {
                    error!("Send channel error. Reason: {:?}", e)
                }
            });
        }
    }
}

/// An UDP-sender, wrapper for the UDP-sending thread.
pub struct UdpSender {
    // socket used for sending packets
    socket: UdpSocket,
}

impl UdpSender {
    /// This will run the udp sender on it's own thread.
    pub fn run(addr: SocketAddr) -> Result<SendHandler> {
        let socket = UdpSocket::bind(&addr, NetworkConfig::default())?;
        let mut udp_sender = UdpSender { socket };

        let (tx, rx) = mpsc::sync_channel(500);

        let thread_handle = thread::spawn(move || {
            udp_sender.start_sending(rx);
        });

        Ok(SendHandler::new(tx, thread_handle))
    }
}

impl PacketSending for UdpSender {
    // 1. Receives a packets from the channel containing packets to send to some endpoint.
    // 2. Sent the packet to a specific client.
    fn start_sending(&mut self, rx: Receiver<ServerSocketEvent>) {
        loop {
            match rx.recv() {
                Ok(packet) => match packet {
                    ServerSocketEvent::Packet(packet) => {
                        if let Err(e) = self.socket.send(&packet) {
                            error!("Something went wrong when trying to send a packet with UDP socket. Reason: {:?}", e)
                        }
                    }
                    _ => warn!("The UDP-sender can only send packets"),
                },
                Err(e) => error!(
                    "Something went wrong when receiving from channel. Reason: {:?}",
                    e
                ),
            }
        }
    }
}
