//! Builds IR command buffers from Frames and feeds them into the backend.

use renderer::ir::CommandBuffer;

/// A trait that describes a renderable Frame element.
pub trait Renderable {
    fn to_cmdbuf(&self) -> CommandBuffer;
}

pub mod lights;
pub mod objects;

mod frontend;
pub use self::frontend::{Frontend, Frame};
