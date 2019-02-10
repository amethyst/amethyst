/// This module is about the communication with this host to some other endpoints.
///
/// 1. Sending Data
/// 2. Receiving Data
/// 3. Broadcasting
use crate::error::Result;
use crate::server::{
    ReceiveHandler, SendHandler, ServerConfig, ServerSocketEvent, UdpReceiver, UdpSender,
};
use laminar::Packet;
use std::sync::{Arc, Mutex};

/// 'Host' abstracts TCP and UDP sockets away.
pub struct Host {
    // Handler to access the internals of the UDP receiving thread
    udp_receiver: Arc<Mutex<ReceiveHandler>>,
    // Handler to access the internals of the UDP sender thread
    udp_sender: Arc<SendHandler>,
}

impl Host {
    /// This will start and return an instance of the host.
    /// 1: Fire up a TCP-sender and TCP-receiver if enabled in config.
    /// 2: Fire up a UDP-sender and UDP-receiver.
    /// 3: Set up some `channels` to communicate with underlying threads.
    ///
    /// The method uses the config provided when creating a `host` instance.
    pub fn run(config: &ServerConfig) -> Result<Host> {
        // setup a UDP-receiver which will receive packets from any endpoint.
        let udp_receiver = Arc::new(Mutex::new(UdpReceiver::run(config.udp_recv_addr, &config)?));

        // setup the UDP-sender which will send packets to an certain endpoint.
        let udp_sender = Arc::new(UdpSender::run(config.udp_send_addr)?);

        Ok(Host {
            udp_sender,
            udp_receiver,
        })
    }

    /// Get the handle to the internals of the UDP-receiving threat.
    pub fn udp_receive_handle(&self) -> Arc<Mutex<ReceiveHandler>> {
        self.udp_receiver.clone()
    }

    /// Get the handle to the internals of the UDP-sending thread.
    pub fn udp_send_handle(&self) -> Arc<SendHandler> {
        self.udp_sender.clone()
    }

    /// Schedule a TCP-packet for sending.
    pub fn send_tcp(&mut self, _payload: &[u8]) -> Result<()> {
        unimplemented!()
    }

    /// Schedule a UDP-packet for sending.
    pub fn send_udp(&mut self, packet: Packet) -> Result<()> {
        self.udp_sender.send(ServerSocketEvent::Packet(packet))?;
        Ok(())
    }
}
