//! The network filter base trait
use super::{ConnectionState, NetConnectionPool, NetEvent};
use std::marker::PhantomData;
use std::net::SocketAddr;

/// Network filter base trait providing an event filtering interface.
pub trait NetFilter<T>: Send + Sync
where
    T: PartialEq,
{
    /// Check if the event is allowed to pass through this filter.
    fn allow(&mut self, pool: &NetConnectionPool, source: &SocketAddr, event: &NetEvent<T>)
        -> bool;
}

/// A filter that checks if the incoming event is from a connected client.
pub struct FilterConnected;


impl<T> NetFilter<T> for FilterConnected
where
    T: PartialEq,
{
    /// Checks if the event is from a connected client.
    fn allow(
        &mut self,
        pool: &NetConnectionPool,
        source: &SocketAddr,
        event: &NetEvent<T>,
    ) -> bool {
        if let Some(ref conn) = pool.connection_from_address(source) {
            if conn.state == ConnectionState::Connected {
                true
            } else /*if conn.state == ConnectionState::Connecting */{
                match *event {
                    NetEvent::Connect { client_uuid } => true,
                    NetEvent::Connected { server_uuid } => true,
                    _ => false,
                }
            }
        } else {
            match *event {
                NetEvent::Connect { client_uuid } => true,
                NetEvent::Connected { server_uuid } => true,
                _ => false,
            }

        }
    }
}
