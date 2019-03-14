//! Provides structures used to load audio files.
//!
use amethyst_assets::{
    Asset, AssetStorage, Handle, Loader, PrefabData, ProcessableAsset, ProcessingState,
};
use amethyst_core::ecs::prelude::{Entity, Read, ReadExpect, VecStorage};
use amethyst_error::Error;

use crate::formats::AudioData;

/// A handle to a source asset.
pub type SourceHandle = Handle<Source>;

/// A loaded audio file
#[derive(Clone)]
pub struct Source {
    /// The bytes of this audio source.
    pub bytes: Vec<u8>,
}

impl AsRef<[u8]> for Source {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl Asset for Source {
    const NAME: &'static str = "audio::Source";
    type Data = AudioData;
    type HandleStorage = VecStorage<SourceHandle>;
}

impl ProcessableAsset for Source {
    fn process(data: AudioData) -> Result<ProcessingState<Source>, Error> {
        Ok(ProcessingState::Loaded(Source { bytes: data.0 }))
    }
}

impl<'a> PrefabData<'a> for AudioData {
    type SystemData = (ReadExpect<'a, Loader>, Read<'a, AssetStorage<Source>>);
    type Result = Handle<Source>;

    fn add_to_entity(
        &self,
        _: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<Handle<Source>, Error> {
        Ok(system_data
            .0
            .load_from_data(self.clone(), (), &system_data.1))
    }
}
