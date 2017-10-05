//! Orientation of objects


/// Orientation struct.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Orientation {
    /// Forward vector [x, y, z]
    pub forward: [f32; 3],
    /// Right vector [x, y, z]
    pub right: [f32; 3],
    /// Up vector [x, y, z]
    pub up: [f32; 3],
}

impl Default for Orientation {
    fn default() -> Self {
        forward:     [1.0, 0.0, 0.0],
        right:       [0.0,-1.0, 0.0],
        up:          [0.0, 0.0, 1.0],
    }
}
