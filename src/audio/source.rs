//! Provides structures used to load audio files.

use std::sync::Arc;

use super::AudioContext;
use assets::*;

/// A loaded audio file
#[derive(Clone)]
pub struct Source {
    pub(crate) pointer: AssetPtr<Arc<Vec<u8>>, Source>,
}

impl AsRef<Arc<Vec<u8>>> for Source {
    fn as_ref(&self) -> &Arc<Vec<u8>> {
        self.pointer.inner_ref()
    }
}

impl AsRef<[u8]> for Source {
    fn as_ref(&self) -> &[u8] {
        &*self.pointer.inner_ref()
    }
}

impl Asset for Source {
    type Context = AudioContext;
}
