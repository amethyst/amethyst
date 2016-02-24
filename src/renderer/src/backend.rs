//! Consumes command buffers and executes them in the correct order.

use gfx::Device;
use gfx::SubmitInfo;

/// Makes low-level graphics API calls and manages memory.
pub struct Backend;

impl Backend {
    /// Creates a new renderer backend.
    pub fn new() -> Backend {
        Backend
    }

    pub fn submit<D: Device>(buf: &SubmitInfo<D>) {
        unimplemented!()
    }
}
