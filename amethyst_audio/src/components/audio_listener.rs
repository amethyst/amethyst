use amethyst_core::math::Point3;

/// An audio listener, add this component to the local player character.
#[derive(Clone, Debug)]
pub struct AudioListener {
    /// Position of the left ear relative to the global transform on this entity.
    pub left_ear: Point3<f32>,
    /// Position of the right ear relative to the global transform on this entity.
    pub right_ear: Point3<f32>,
}

impl Default for AudioListener {
    fn default() -> Self {
        AudioListener {
            left_ear: Point3::new(-1.0, 0.0, 0.0),
            right_ear: Point3::new(1.0, 0.0, 0.0),
        }
    }
}
