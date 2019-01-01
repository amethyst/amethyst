mod config;
mod host;
mod receive_handler;
mod send_handler;
mod server_socket_event;
mod udp;

pub use self::config::ServerConfig;
pub use self::host::Host;
pub use self::receive_handler::ReceiveHandler;
pub use self::send_handler::SendHandler;
pub use self::udp::{UdpReceiver, UdpSender};

pub use self::server_socket_event::{ClientEvent, ServerSocketEvent};
use std::sync::mpsc::{Receiver, Sender};

/// Can be implemented for the receiving side of a socket.
pub trait PacketReceiving {
    /// Start receiving data.
    /// You have to pass the `Sender` side of a channel to this function so that the `Receiver` side of the channel can read all received packets send on this `Sender` by this function.
    fn start_receiving(&mut self, tx: Sender<ServerSocketEvent>);
}

/// Can be implemented for the sending side of a socket.
pub trait PacketSending {
    /// This will send all data.
    /// You have to pass the `Receiver` side of a channel to this function so that packets send on the `Sender` could be read by this function.
    fn start_sending(&mut self, rx: Receiver<ServerSocketEvent>);
}
