//! Color value types.

use amethyst_core::specs::{Component, DenseVecStorage};

use gfx::shade::{Formatted, ToUniform};
use gfx_core::shade::{BaseType, ContainerType, UniformValue};
use glsl_layout::{vec3, vec4};
use serde::{Deserialize, Serialize};

/// An RGBA color value.
///
/// ## As a Component
/// If you attach this as a component to an entity then passes should multiply any rendered pixels
/// in the component with this color.  Please note alpha multiplication will only produce
/// transparency if the rendering pass would normally be capable of rendering that entity
/// transparently.
///
/// ## More than a Component
/// This structure has more uses than just as a component, and you'll find it in other places
/// throughout the `amethyst_renderer` API.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
pub struct Rgba(pub f32, pub f32, pub f32, pub f32);

impl Rgba {
    /// Solid black color value.
    pub const BLACK: Rgba = Rgba(0.0, 0.0, 0.0, 1.0);
    /// Solid blue color value.
    pub const BLUE: Rgba = Rgba(0.0, 0.0, 1.0, 1.0);
    /// Solid green color value.
    pub const GREEN: Rgba = Rgba(0.0, 1.0, 0.0, 1.0);
    /// Solid red color value.
    pub const RED: Rgba = Rgba(1.0, 0.0, 0.0, 1.0);
    /// Transparent color value.
    pub const TRANSPARENT: Rgba = Rgba(0.0, 0.0, 0.0, 0.0);
    /// Solid white color value.
    pub const WHITE: Rgba = Rgba(1.0, 1.0, 1.0, 1.0);

    /// Returns a solid black color value.
    pub fn black() -> Rgba {
        Rgba::BLACK
    }

    /// Returns a solid blue color value.
    pub fn blue() -> Rgba {
        Rgba::BLUE
    }

    /// Returns a solid green color value.
    pub fn green() -> Rgba {
        Rgba::GREEN
    }

    /// Returns a solid red color value.
    pub fn red() -> Rgba {
        Rgba::RED
    }

    /// Returns a transparent color value.
    pub fn transparent() -> Rgba {
        Rgba::TRANSPARENT
    }

    /// Returns a solid white color value.
    pub fn white() -> Rgba {
        Rgba::WHITE
    }
}

impl Default for Rgba {
    fn default() -> Rgba {
        Rgba::black()
    }
}

impl Component for Rgba {
    type Storage = DenseVecStorage<Self>;
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

impl From<Rgba> for vec3 {
    fn from(Rgba(r, g, b, _): Rgba) -> vec3 {
        [r, g, b].into()
    }
}

impl From<Rgba> for vec4 {
    fn from(Rgba(r, g, b, a): Rgba) -> vec4 {
        [r, g, b, a].into()
    }
}
