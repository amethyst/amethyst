//! Provides structures used to load audio files.
//!
use std::result::Result as StdResult;

use amethyst_assets::{
    Asset, AssetStorage, Handle, Loader, PrefabData, PrefabError, ProcessingState, Result,
};
use amethyst_core::specs::prelude::{Entity, Read, ReadExpect, VecStorage};

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

impl Into<Result<ProcessingState<Source>>> for AudioData {
    fn into(self) -> Result<ProcessingState<Source>> {
        Ok(ProcessingState::Loaded(Source { bytes: self.0 }))
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
    ) -> StdResult<Handle<Source>, PrefabError> {
        Ok(system_data
            .0
            .load_from_data(self.clone(), (), &system_data.1))
    }
}
