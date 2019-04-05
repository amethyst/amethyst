//! `amethyst` rendering ecs resources
//!

use amethyst_assets::PrefabData;
use amethyst_core::ecs::{Component, DenseVecStorage, Entity, Write};
use amethyst_error::Error;

/// The ambient color of a scene
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AmbientColor(pub palette::Srgba);



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
    ) -> Result<(), Error> {
        ambient.0 = self.0;
        Ok(())
    }
}

/// A single object tinting applied in multiplicative mode (modulation)
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Tint(pub palette::Srgba);

impl Component for Tint {
    type Storage = DenseVecStorage<Self>;
}
