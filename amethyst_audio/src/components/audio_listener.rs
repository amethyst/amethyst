use amethyst_core::{
    ecs::{prelude::Component, storage::HashMapStorage},
    math::{one, zero, Point3, RealField},
};

/// An audio listener, add this component to the local player character.
#[derive(Debug)]
pub struct AudioListener<N: RealField> {
    /// Position of the left ear relative to the global transform on this entity.
    pub left_ear: Point3<N>,
    /// Position of the right ear relative to the global transform on this entity.
    pub right_ear: Point3<N>,
}

impl<N: RealField> Default for AudioListener<N> {
    fn default() -> Self {
        AudioListener {
            left_ear: Point3::new(-one::<N>(), zero::<N>(), zero::<N>()),
            right_ear: Point3::new(one::<N>(), zero::<N>(), zero::<N>()),
        }
    }
}

impl<N: RealField> Component for AudioListener<N> {
    type Storage = HashMapStorage<Self>;
}
