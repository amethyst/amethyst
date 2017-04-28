//! Color value types.

use gfx::shade::{Formatted, ToUniform};
use gfx_core::shade::{BaseType, ContainerType, UniformValue};

/// An RGBA color value.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
pub struct Rgba(pub f32, pub f32, pub f32, pub f32);

impl Rgba {
    /// Returns a solid black color value.
    pub fn black() -> Rgba {
        Rgba(0.0, 0.0, 0.0, 1.0)
    }

    /// Returns a solid blue color value.
    pub fn blue() -> Rgba {
        Rgba(0.0, 0.0, 1.0, 1.0)
    }

    /// Returns a solid green color value.
    pub fn green() -> Rgba {
        Rgba(0.0, 1.0, 0.0, 1.0)
    }

    /// Returns a solid red color value.
    pub fn red() -> Rgba {
        Rgba(1.0, 0.0, 0.0, 1.0)
    }

    /// Returns a transparent color value.
    pub fn transparent() -> Rgba {
        Rgba(0.0, 0.0, 0.0, 0.0)
    }

    /// Returns a solid white color value.
    pub fn white() -> Rgba {
        Rgba(1.0, 1.0, 1.0, 1.0)
    }
}

impl Default for Rgba {
    fn default() -> Rgba {
        Rgba::white()
    }
}

impl From<[f32; 3]> for Rgba {
    fn from(arr: [f32; 3]) -> Rgba {
        Rgba(arr[0], arr[1], arr[2], 1.0)
    }
}

impl From<[f32; 4]> for Rgba {
    fn from(arr: [f32; 4]) -> Rgba {
        Rgba(arr[0], arr[1], arr[2], arr[3])
    }
}

impl From<(f32, f32, f32)> for Rgba {
    fn from((r, g, b): (f32, f32, f32)) -> Rgba {
        Rgba(r, g, b, 1.0)
    }
}

impl From<(f32, f32, f32, f32)> for Rgba {
    fn from((r, g, b, a): (f32, f32, f32, f32)) -> Rgba {
        Rgba(r, g, b, a)
    }
}

impl From<Rgba> for [f32; 3] {
    fn from(Rgba(r, g, b, _): Rgba) -> [f32; 3] {
        [r, g, b]
    }
}

impl From<Rgba> for [f32; 4] {
    fn from(Rgba(r, g, b, a): Rgba) -> [f32; 4] {
        [r, g, b, a]
    }
}

impl From<Rgba> for (f32, f32, f32) {
    fn from(Rgba(r, g, b, _): Rgba) -> (f32, f32, f32) {
        (r, g, b)
    }
}

impl From<Rgba> for (f32, f32, f32, f32) {
    fn from(Rgba(r, g, b, a): Rgba) -> (f32, f32, f32, f32) {
        (r, g, b, a)
    }
}

impl Formatted for Rgba {
    fn get_format() -> (BaseType, ContainerType) {
        (BaseType::F32, ContainerType::Vector(4))
    }
}

impl ToUniform for Rgba {
    fn convert(self) -> UniformValue {
        UniformValue::F32Vector4(self.into())
    }
}
