//! Types of light sources.

/// An area light.
pub struct Area;

/// A directional light.
pub struct Directional {
    pub color: [f32; 4],
    pub direction: [f32; 3],
    pub intensity: f32,
}

/// A point light.
pub struct Point {
    pub color: [f32; 4],
    pub intensity: f32,
    pub location: [f32; 3],
}

/// A spot light.
pub struct Spot {
    pub angle: f32,
    pub color: [f32; 4],
    pub direction: [f32; 3],
    pub intensity: f32,
    pub location: [f32; 3],
}
