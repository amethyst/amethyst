use super::{ConnectionManagerSystem, NetConnectionPool, NetIdentity, NetReceiveBuffer,
            NetSendBuffer, NetSocketSystem};
use amethyst_core::bundle::{ECSBundle, Result};
use filter::NetFilter;
use serde::Serialize;
use serde::de::DeserializeOwned;
use shred::DispatcherBuilder;
use specs::World;
use std::net::SocketAddr;
use uuid::Uuid;

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
    pub fn new_client(
        ip: &'a str,
        port: Option<u16>,
        filters: Vec<Box<NetFilter<T>>>,
    ) -> Self {
        NetworkBundle {
            ip,
            port,
            filters,
            is_server: false,
            connect_to: None,
        }
    }

    /// Creates a new NetworkBundle in server mode
    pub fn new_server(
        ip: &'a str,
        port: Option<u16>,
        filters: Vec<Box<NetFilter<T>>>,
    ) -> Self {
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

impl<'a, 'b, 'c, T> ECSBundle<'a, 'b> for NetworkBundle<'c, T>
where
    T: Send + Sync + PartialEq + Serialize + Clone + DeserializeOwned + 'static,
{
    fn build(
        mut self,
        world: &mut World,
        mut builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        let mut pool = NetConnectionPool::new();
        world.add_resource(NetSendBuffer::<T>::new());
        world.add_resource(NetReceiveBuffer::<T>::new());

        let uuid = Uuid::new_v4();

        let custom_port = self.port.is_some();

        if !custom_port {
            self.port = Some(0);
            info!("Starting NetworkBundle using a random port.");
        }

        let mut s = NetSocketSystem::<T>::new(self.ip, self.port.unwrap(), self.filters)
            .expect("Failed to open network system.");
        if let Some(c) = self.connect_to {
            s.connect(c, &mut pool, uuid);
        }

        world.add_resource(pool);
        world.add_resource(NetIdentity { uuid });

        builder = builder.add(s, "net_socket", &[]);
        builder = builder.add(
            ConnectionManagerSystem::<T>::new(self.is_server),
            "connection_manager",
            &["net_socket"],
        );

        Ok(builder)
    }
}
