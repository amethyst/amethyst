//! Platform-agnostic intermediate representation used by the frontend and
//! backend to communicate.

pub mod state_dynamic;
pub mod state_static;

mod cmd_buffer;
mod cmd_encoder;
mod cmd_queue;

pub use self::cmd_buffer::{Command, CommandBuffer};
pub use self::cmd_encoder::CommandEncoder;
pub use self::cmd_queue::CommandQueue;
