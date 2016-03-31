//! Consumes command buffers and executes them in the correct order.

use std::any::Any;
use gfx::{CommandBuffer, Resources};

/// Makes low-level graphics API calls and manages memory.
pub struct Backend;

impl Backend {
    /// Creates a new renderer backend.
    pub fn new() -> Backend {
        Backend
    }

    pub fn submit<R: Resources, T: Any + CommandBuffer<R>>(buf: Box<T>) {
        unimplemented!()
    }
}
