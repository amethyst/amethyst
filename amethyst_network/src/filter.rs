//! The network filter base trait

use std::{marker::PhantomData, net::SocketAddr};

use crate::NetEvent;

/// Network filter base trait providing an event filtering interface.
pub trait NetFilter<T>: Send + Sync
where
    T: PartialEq,
{
    /// Check if the event is allowed to pass through this filter.
    fn allow(&mut self, source: &SocketAddr, event: &NetEvent<T>) -> bool;
}

/// A filter that checks if the incoming event is from a connected client.
pub struct FilterConnected<T> {
    _pd: PhantomData<T>,
}

impl<T> FilterConnected<T> {
    /// Creates a new FilterConnected filter.
    pub fn new() -> Self {
        FilterConnected { _pd: PhantomData }
    }
}

impl<T> NetFilter<T> for FilterConnected<T>
where
    T: PartialEq + Send + Sync,
{
    /// Checks if the event is from a connected client.
    /// Note: This is not usable currently.
    fn allow(&mut self, _source: &SocketAddr, event: &NetEvent<T>) -> bool {
        match event {
            NetEvent::Connect { .. } => true,
            NetEvent::Connected { .. } => true,
            _ => false,
        }
    }
}
