
use filters::filter::*;
use resources::{NetConnection};
use std::net::SocketAddr;
use std::marker::PhantomData;

/// A filter that checks if the incoming event is from a connected client.
pub struct FilterConnected<T>{
    net_event_types:PhantomData<T>,
}

impl<T> NetFilter<T>{
    /// Checks if the event is from a connected client.
    pub fn allow(&mut self,_remotes:Vec<NetConnection>,_source:SocketAddr,_event:T)->bool{
        true //TODO
    }
}