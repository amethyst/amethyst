//! Module containing various utilities to run a client/server-based network simulation. Expect
//! more utilities to make their way into this module. e.g. "Component synchronization",
//! "Matchmaking", etc.

mod client;
mod events;
mod message;
mod requirements;
mod resource;
mod timing;
mod transport;

pub use events::NetworkSimulationEvent;
pub use message::Message;
pub use requirements::{DeliveryRequirement, UrgencyRequirement};
pub use resource::NetworkSimulationResource;
pub use timing::{NetworkSimulationTime, NetworkSimulationTimeSystem};
pub use transport::laminar;
pub use transport::udp;
