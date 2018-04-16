extern crate rand;

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
use rand::Rng;

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
    /// Creates a new NetworkClientBundle
    pub fn new(
        ip: &'a str,
        port: Option<u16>,
        filters: Vec<Box<NetFilter<T>>>,
        is_server: bool,
    ) -> Self {
        NetworkBundle {
            ip,
            port,
            filters,
            is_server,
            connect_to: None,
        }
    }
    /// Automatically connects to the specified client  network socket address.
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


        // TODO: If the port is already in use, try another random port

        if !custom_port{
            // [1025–65535]
            self.port = Some(rand::thread_rng().gen_range(1025, 65535 + 1) as u16)
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
