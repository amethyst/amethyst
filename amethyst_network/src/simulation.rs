//! Module containing various utilities to run a client/server-based network simulation. Expect
//! more utilities to make their way into this module. e.g. "Component synchronization",
//! "Matchmaking", etc.

mod events;
mod message;
mod requirements;
mod timing;
mod transport;

pub use events::NetworkSimulationEvent;
pub use message::Message;
pub use requirements::{DeliveryRequirement, UrgencyRequirement};
pub use timing::NetworkSimulationTime;
pub use transport::{laminar, tcp, udp, TransportResource};
