extern crate rand;

use systems::*;
use resources::*;
use amethyst_core::bundle::{ECSBundle, Result};
use specs::World;
use shred::DispatcherBuilder;
use std::marker::PhantomData;
use serde::Serialize;
use serde::de::DeserializeOwned;
use filter::NetFilter;

pub struct NetworkClientBundle<'a,T> {
    ip: &'a str,
    port: Option<u16>,
    filters: Vec<Box<NetFilter<T>>>,
}

impl<'a,T> NetworkClientBundle<'a,T> where T: Send+Sync{
    /// Creates a new NetworkClientBundle
    pub fn new(ip: &'a str,port: Option<u16>,filters: Vec<Box<NetFilter<T>>>) -> Self {
        NetworkClientBundle {
            ip,
            port,
            filters,
        }
    }
}

impl<'a, 'b, 'c, T> ECSBundle<'a, 'b> for NetworkClientBundle<'c,T> where T: Send+Sync+PartialEq+Serialize+Clone+DeserializeOwned+'static {
    fn build(
        mut self,
        world: &mut World,
        mut builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        world.add_resource(NetConnectionPool::new());
        world.add_resource(NetSendBuffer::<T>::new());
        world.add_resource(NetReceiveBuffer::<T>::new());

        while self.port.is_none() || self.port.unwrap() < 200 {
            self.port = Some(rand::random::<u16>());
        }

        builder = builder.add(NetClientSystem::<T>::new(self.ip,self.port.unwrap(),self.filters).expect("Failed to open network client system."),"net_client",&[]);

        Ok(builder)
    }
}