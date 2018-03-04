extern crate rand;

use amethyst_core::bundle::{ECSBundle, Result};
use specs::World;
use shred::DispatcherBuilder;
use std::marker::PhantomData;
use serde::Serialize;
use serde::de::DeserializeOwned;
use filter::NetFilter;
use std::net::SocketAddr;
use uuid::Uuid;
use super::{NetSocketSystem,ConnectionManagerSystem,NetSendBuffer,NetReceiveBuffer,NetConnectionPool,NetIdentity};

pub struct NetworkClientBundle<'a,T> {
    ip: &'a str,
    port: Option<u16>,
    filters: Vec<Box<NetFilter<T>>>,
    is_server: bool,
    connect_to: Option<SocketAddr>,
}

impl<'a,T> NetworkClientBundle<'a,T>{
    /// Creates a new NetworkClientBundle
    pub fn new(ip: &'a str,port: Option<u16>,filters: Vec<Box<NetFilter<T>>>,is_server: bool) -> Self {
        NetworkClientBundle {
            ip,
            port,
            filters,
            is_server,
            connect_to: None,
        }
    }
    pub fn with_connect(mut self,socket: SocketAddr) -> Self{
        self.connect_to = Some(socket);
        self
    }
}

impl<'a, 'b, 'c, T> ECSBundle<'a, 'b> for NetworkClientBundle<'c,T> where T: Send+Sync+PartialEq+Serialize+Clone+DeserializeOwned+'static {
    fn build(
        mut self,
        world: &mut World,
        mut builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        let mut pool = NetConnectionPool::new();
        world.add_resource(NetSendBuffer::<T>::new());
        world.add_resource(NetReceiveBuffer::<T>::new());

        let uuid = Uuid::new_v4();

        while self.port.is_none() || self.port.unwrap() < 200 {
            self.port = Some(rand::random::<u16>());
        }
        let mut s = NetSocketSystem::<T>::new(self.ip,self.port.unwrap(),self.filters).expect("Failed to open network system.");
        if let Some(c) = self.connect_to{
            s.connect(c,&mut pool,uuid);
        }

        world.add_resource(pool);
        world.add_resource(NetIdentity{
            uuid,
        });

        builder = builder.add(s,"net_socket",&[]);
        builder = builder.add(ConnectionManagerSystem::<T>::new(self.is_server),"connection_manager",&["net_socket"]);

        Ok(builder)
    }
}