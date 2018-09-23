//! Provides a client-server networking architecture to amethyst.

#![warn(missing_docs)]

extern crate amethyst_assets;
extern crate amethyst_core;
extern crate bincode;
#[macro_use]
extern crate log;
extern crate mio;
extern crate rand;
#[macro_use]
extern crate serde;
extern crate fern;
extern crate shred;
extern crate shrev;
extern crate uuid;

mod bundle;
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
pub use utils::{deserialize_event, send_event /*send_to, send_to_all, send_to_all_except*/};
