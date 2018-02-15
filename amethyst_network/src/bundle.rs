extern crate rand;

use systems::*;
use resources::*;
use amethyst_core::bundle::{ECSBundle, Result};
use specs::{DispatcherBuilder, World};
use std::marker::PhantomData;

pub struct NetworkClientBundle<'a,T> {
    ip: &'a str,
    port: Option<u16>,
    _marker: PhantomData<T>,
}

impl<'a,T> NetworkClientBundle<'a,T> {
    /// Creates a new NetworkClientBundle
    pub fn new(ip: &str,port: Option<u16>) -> Self {
        NetworkClientBundle {
            ip,
            port,
            _marker: PhantomData,
        }
    }
}

impl<'a, 'b, 'c, T> ECSBundle<'a, 'b> for NetworkClientBundle<'c,T> {
    fn build(
        self,
        world: &mut World,
        mut builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        world.add_resource(NetConnectionPool::new());
        world.add_resource(NetSendBuffer::<T>::new());
        world.add_resource(NetReceiveBuffer::<T>::new());

        while self.port.is_none() || self.port.unwrap() < 200 {
            self.port = Some(rand::random::<u32>());
        }

        builder = builder.add(NetClientSystem::<T>::new(self.ip,self.port.unwrap()),"net_client",&[]);

        Ok(builder)
    }
}