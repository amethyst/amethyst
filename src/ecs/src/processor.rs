//! Performs actions based on the relevant components found in the game world.

/// The error type reported by processors if they fail to initialize.
/// TODO: Implement actual error handling either with Result or custom
/// type alias.
pub struct ProcessorError;

/// The trait implemented by all processors.
pub trait Processor {
    /// TODO: Need to finalize API design here, according to [issue #10].
    /// 
    /// [issue #10]: https://github.com/ebkalderon/amethyst/issues/10
    fn process(&mut self);
}
