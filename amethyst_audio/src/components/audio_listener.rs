use specs::{Component, HashMapStorage};

use output::Output;

/// An audio listener, add this component to the local player character.
#[derive(Debug)]
pub struct AudioListener {
    /// Output used by this listener to emit sounds to
    pub output: Output,
    /// Position of the left_ear relative to the global transform on this entity.
    pub left_ear: [f32; 3],
    /// Position of the right ear relative to the global transform on this entity.
    pub right_ear: [f32; 3],
}

impl Component for AudioListener {
    type Storage = HashMapStorage<Self>;
}
