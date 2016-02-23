//! Builds command buffers from frames and feeds them into the backend.

use frame::Frame;
use pipeline::Pipeline;

/// A simple renderer frontend. Accepts a `Pipeline` on startup, and parses
/// `Frame`s.
pub struct Frontend {
    pipe: Pipeline,
}

impl Frontend {
    /// Creates a new renderer frontend.
    pub fn new(pipe: Pipeline) -> Frontend {
        Frontend {
            pipe: pipe,
        }
    }

    /// Draws a frame with the currently set render pipeline.
    ///
    /// TODO: Build actual modular, parallelized Object translators.
    pub fn draw(&mut self, _frame: Frame) {
        unimplemented!();
    }
}
