use amethyst_core::{
    ecs::{prelude::Component, storage::HashMapStorage},
    math::{one, zero, Point3},
    Float,
};

/// An audio listener, add this component to the local player character.
#[derive(Debug)]
pub struct AudioListener {
    /// Position of the left ear relative to the global transform on this entity.
    pub left_ear: Point3<Float>,
    /// Position of the right ear relative to the global transform on this entity.
    pub right_ear: Point3<Float>,
}

impl Default for AudioListener {
    fn default() -> Self {
        AudioListener {
            left_ear: Point3::new(-one::<Float>(), zero::<Float>(), zero::<Float>()),
            right_ear: Point3::new(one::<Float>(), zero::<Float>(), zero::<Float>()),
        }
    }
}

impl Component for AudioListener {
    type Storage = HashMapStorage<Self>;
}
