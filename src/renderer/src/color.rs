//! Color value types.

/// An RGBA color value.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
pub struct Rgba(pub f32, pub f32, pub f32, pub f32);

impl Default for Rgba {
    fn default() -> Rgba {
        Rgba(1.0, 1.0, 1.0, 1.0)
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
