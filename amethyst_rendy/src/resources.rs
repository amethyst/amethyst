//! `amethyst` rendering ecs resources
//!

/// The ambient color of a scene
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AmbientColor(#[serde(with = "crate::serde_shim::srgba")] pub palette::Srgba);

impl AsRef<palette::Srgba> for AmbientColor {
    fn as_ref(&self) -> &palette::Srgba {
        &self.0
    }
}

/// A single object tinting applied in multiplicative mode (modulation)
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Tint(#[serde(with = "crate::serde_shim::srgba")] pub palette::Srgba);

impl Into<[f32; 4]> for Tint {
    fn into(self) -> [f32; 4] {
        let (r, g, b, a) = self.0.into_components();
        [r, g, b, a]
    }
}
