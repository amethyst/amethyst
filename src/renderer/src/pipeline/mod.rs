//! Encodes information about how to draw the scene.

mod stage;

pub use self::stage::{Stage, Step};

/// A set of stages that describes how to draw a frame.
#[derive(Debug)]
pub struct Pipeline {
    name: String,
    stages: Vec<Stage>,
}

impl Pipeline {
    /// Creates an empty pipeline and assigns it a descriptive name.
    pub fn new(name: &str) -> Pipeline {
        Pipeline {
            name: name.to_string(),
            stages: Vec::new(),
        }
    }
}
