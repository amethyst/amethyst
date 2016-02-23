//! Performs actions based on the relevant components found in the game world.

/// The error type reported by SimBuilder if they fail to initialize.
/// TODO: original note specified it was en error type reported by a **processor**,
/// although, as seen below, Processor doesn't have any function to return an error,
/// thus, only SimBuilder can return Result as of now.
pub type ProccessorResult = Result<(), String>;

/// The trait implemented by all processors.
pub trait Processor {
    /// TODO: Need to finalize API design here, according to [issue #10].
    ///
    /// [issue #10]: https://github.com/ebkalderon/amethyst/issues/10
    fn process(&mut self);
}
