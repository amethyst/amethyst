//! This module holds the underlying system implementations for each of the various transport
//! protocols. One important thing to note if you're implementing your own, the underlying sockets
//! MUST be non-blocking in order to play nicely with the ECS scheduler.

mod resource;

pub mod laminar;
pub mod tcp;
pub mod udp;

pub use resource::TransportResource;

const NETWORK_SIM_TIME_SYSTEM_NAME: &str = "simulation_time";
const NETWORK_SEND_SYSTEM_NAME: &str = "network_send";
const NETWORK_RECV_SYSTEM_NAME: &str = "network_recv";
const NETWORK_POLL_SYSTEM_NAME: &str = "network_poll";
