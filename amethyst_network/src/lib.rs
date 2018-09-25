//! Provides a client-server networking architecture to amethyst.

#![warn(missing_docs)]

extern crate amethyst_core;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
extern crate fern;
extern crate shred;
extern crate shrev;
extern crate uuid;

mod bundle;
// TODO: Uncomment components and entity sync
//mod components;
mod connection;
mod filter;
mod net_event;
mod network_socket;
mod test;
mod utils;

pub use bundle::NetworkBundle;
pub use connection::{ConnectionState, NetConnection, NetIdentity};
pub use filter::{FilterConnected, NetFilter};
pub use net_event::NetEvent;
pub use network_socket::NetSocketSystem;
pub use utils::{deserialize_event, send_event};
