//! The network filter base trait
use resources::*;
use std::net::SocketAddr;
use std::marker::PhantomData;

/// Network filter base trait providing an event filtering interface.
pub trait NetFilter<T>: Send+Sync where T: Send+Sync+PartialEq{
    /// Check if the event is allowed to pass through this filter.
    fn allow(&mut self,pool:&NetConnectionPool,source:&SocketAddr,event:&NetEvent<T>)->bool;
}

/// A filter that checks if the incoming event is from a connected client.
pub struct FilterConnected;

impl<T> NetFilter<T> for FilterConnected where T: Send+Sync+PartialEq+Sized{
    /// Checks if the event is from a connected client.
    fn allow(&mut self,pool:&NetConnectionPool,source:&SocketAddr,event:&NetEvent<T>)->bool{
        if let Some(ref conn) = pool.connection_from_address(source){
            if conn.state == ConnectionState::Connected{
                return true
            }else if conn.state == ConnectionState::Connecting{
                if *event == NetEvent::Connect || *event == NetEvent::Connected{
                    return true
                }
            }
        }
        false
    }
}
