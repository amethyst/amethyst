//! Builds command buffers from frames and feeds them into the backend.

use backend::Backend;
use frame::Frame;
use pipeline::Pipeline;

/// A simple renderer frontend. Accepts a `Pipeline` on startup, and parses
/// `Frame`s.
pub struct Renderer {
    back: Backend,
    pipe: Pipeline,
}

impl Renderer {
    /// Creates a new rendering engine.
    ///
    /// TODO: Decide whether the backend should be initialized at creation time
    /// or at a different time. If at creation time, this method should return
    /// `Result<Renderer, RendererError>` with `RendererError` implementing the
    /// `Error` trait.
    pub fn new(pipe: Pipeline) -> Renderer {
        Renderer {
            back: Backend::new(),
            pipe: pipe,
        }
    }

    /// Creates a new rendering engine with a deferred pipeline.
    pub fn new_deferred() -> Renderer {
        unimplemented!();
    }

    /// Creates a new rendering engine with a forward pipeline.
    pub fn new_forward() -> Renderer {
        unimplemented!();
    }

    /// Draws a frame with the currently set render pipeline.
    ///
    /// TODO: Build actual modular, parallelized Object translators.
    pub fn draw(&mut self, _frame: Frame) {
        unimplemented!();
    }
}
