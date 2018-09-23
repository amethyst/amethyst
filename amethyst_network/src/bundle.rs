use super::NetSocketSystem;
use amethyst_core::bundle::{Result, SystemBundle};
use amethyst_core::shred::DispatcherBuilder;
use filter::NetFilter;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::net::SocketAddr;

/// A convenience bundle to create the infrastructure needed to send and receive network messages.
pub struct NetworkBundle<'a, T> {
    /// The local ip to bind to.
    ip: &'a str,
    /// The local port to bind to.
    port: Option<u16>,
    /// The filters applied on received network events.
    filters: Vec<Box<NetFilter<T>>>,
    /// Indicates if this should behaves as a server or as a client when handling remote connections.
    is_server: bool,
    /// The server to automatically connect to.
    /// You would usually want this if you set is_server = false.
    connect_to: Option<SocketAddr>,
}

impl<'a, T> NetworkBundle<'a, T> {
    /// Creates a new NetworkBundle in client mode.
    pub fn new_client(ip: &'a str, port: Option<u16>, filters: Vec<Box<NetFilter<T>>>) -> Self {
        NetworkBundle {
            ip,
            port,
            filters,
            is_server: false,
            connect_to: None,
        }
    }

    /// Creates a new NetworkBundle in server mode
    pub fn new_server(ip: &'a str, port: Option<u16>, filters: Vec<Box<NetFilter<T>>>) -> Self {
        NetworkBundle {
            ip,
            port,
            filters,
            is_server: true,
            connect_to: None,
        }
    }

    /// Automatically connects to the specified client network socket address.
    pub fn with_connect(mut self, socket: SocketAddr) -> Self {
        self.connect_to = Some(socket);
        self
    }
}

impl<'a, 'b, 'c, T> SystemBundle<'a, 'b> for NetworkBundle<'c, T>
where
    T: Send + Sync + PartialEq + Serialize + Clone + DeserializeOwned + 'static,
{
    fn build(mut self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        let custom_port = self.port.is_some();

        if !custom_port {
            self.port = Some(0);
            info!("Starting NetworkBundle using a random port.");
        }

        let s = NetSocketSystem::<T>::new(self.ip, self.port.unwrap())
            .expect("Failed to open network system.");

        builder.add(s, "net_socket", &[]);

        Ok(())
    }
}
