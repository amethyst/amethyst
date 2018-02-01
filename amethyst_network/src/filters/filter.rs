//! The network filter base trait

use resources::*;
use std::net::SocketAddr;

/// Network filter base trait providing an event filtering interface.
pub trait NetFilter<T>{
    /// Check if the event is allowed to pass through this filter.
    fn allow(&mut self,remotes:Vec<NetConnection>,source:SocketAddr,event:T)->bool;
}