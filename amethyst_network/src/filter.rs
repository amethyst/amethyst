//! The network filter base trait
use resources::*;
use std::net::SocketAddr;
use std::marker::PhantomData;

/// Network filter base trait providing an event filtering interface.
pub trait NetFilter<T>{
    /// Check if the event is allowed to pass through this filter.
    fn allow(&mut self,pool:&NetConnectionPool,source:&SocketAddr,event:&T)->bool;
}

/// A filter that checks if the incoming event is from a connected client.
pub struct FilterConnected<T>{
    net_event_types:PhantomData<T>,
}

impl<T> NetFilter<T>{
    /// Checks if the event is from a connected client.
    pub fn allow(&mut self,pool:&NetConnectionPool,source:&SocketAddr,event:&T)->bool{
        if let Some(&conn) = pool.connection_from_address(source){
            if conn.state == ConnectionState::Connected{
                return true
            }else if conn.state == ConnectionState::Connecting{
                if event == NetEvent<T>::Connect || event == NetEvent<T>::Connected{
                    return true
                }
            }
        }
        false
    }
}