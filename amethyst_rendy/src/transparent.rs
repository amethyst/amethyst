use amethyst_assets::PrefabData;
use amethyst_core::ecs::{prelude::Component, storage::NullStorage, Entity, WriteStorage};
use amethyst_error::Error;

/// Transparent mesh component
#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct TransparentComponent;

impl Component for TransparentComponent {
    type Storage = NullStorage<Self>;
}

impl<'a> PrefabData<'a> for TransparentComponent {
    type SystemData = WriteStorage<'a, TransparentComponent>;
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        storage: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        storage.insert(entity, TransparentComponent)?;
        Ok(())
    }
}
