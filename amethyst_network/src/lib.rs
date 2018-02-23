//! Provides a client-server networking architecture to amethyst.

//#![deny(missing_docs)]

extern crate amethyst_assets;
extern crate amethyst_core;
#[macro_use]
extern crate serde;
extern crate bincode;
extern crate shrev;
#[macro_use]
extern crate log;
extern crate specs;
extern crate uuid;
extern crate rand;
extern crate shred;

mod bundle;
mod connection;
mod connection_manager;
mod filter;
mod net_event;
mod network_socket;
mod utils;

pub use bundle::NetworkClientBundle;
pub use connection::{NetConnectionPool,NetConnection,NetReceiveBuffer,NetSendBuffer,NetOwner,ConnectionState};
pub use connection_manager::ConnectionManagerSystem;
pub use filter::{FilterConnected,NetFilter};
pub use net_event::{NetEvent,NetSourcedEvent};
pub use network_socket::NetSocketSystem;
pub use utils::{deserialize_event,send_event,send_to};