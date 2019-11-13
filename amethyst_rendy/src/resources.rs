//! `amethyst` rendering ecs resources
//!

use amethyst_assets::PrefabData;
use amethyst_core::ecs::{Component, DenseVecStorage, Entity, Write};
use amethyst_error::Error;

/// The ambient color of a scene
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AmbientColor(#[serde(with = "crate::serde_shim::srgba")] pub palette::Srgba);

impl AsRef<palette::Srgba> for AmbientColor {
    fn as_ref(&self) -> &palette::Srgba {
        &self.0
    }
}

impl<'a> PrefabData<'a> for AmbientColor {
    type SystemData = Write<'a, AmbientColor>;
    type Result = ();

    fn add_to_entity(
        &self,
        _: Entity,
        ambient: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        ambient.0 = self.0;
        Ok(())
    }
}

/// A single object tinting applied in multiplicative mode (modulation)
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Tint(#[serde(with = "crate::serde_shim::srgba")] pub palette::Srgba);

impl Component for Tint {
    type Storage = DenseVecStorage<Self>;
}

impl Into<[f32; 4]> for Tint {
    fn into(self) -> [f32; 4] {
        let (r, g, b, a) = self.0.into_components();
        [r, g, b, a]
    }
}
