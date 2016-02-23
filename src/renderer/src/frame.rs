//! Encodes the lights, objects, and uniform values needed to draw a single
//! frame.

/// A structure holding frame-specific data that is consumed by the frontend.
pub struct Frame;

impl Frame {
    /// Creates an empty frame.
    pub fn new() -> Frame {
        Frame
    }

   /// Creates an initialized frame using the [builder pattern][bp].
   ///
   /// [bp]: https://doc.rust-lang.org/book/method-syntax.html#builder-pattern
    pub fn build() -> FrameBuilder {
        FrameBuilder::new()
    }
}

/// Consuming builder for easily constructing a new frame.
pub struct FrameBuilder {
    frame: Frame,
}

impl FrameBuilder {
    /// Starts building a new frame.
    pub fn new() -> FrameBuilder {
        FrameBuilder { frame: Frame::new() }
    }

    /// Returns the newly-built frame.
    pub fn done(self) -> Frame {
        self.frame
    }
}
