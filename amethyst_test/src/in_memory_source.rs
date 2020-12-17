use std::collections::HashMap;

use amethyst::{assets::Source, error::format_err, Error};
use derive_deref::{Deref, DerefMut};
use derive_new::new;

/// Identifies the in-memory asset source.
pub const IN_MEMORY_SOURCE_ID: &str = "in_memory_asset_source";

/// In-memory implementation of an asset `Source`, purely for tests.
#[derive(Debug, Deref, DerefMut, new)]
pub struct InMemorySource(#[new(default)] pub HashMap<String, Vec<u8>>);

impl Source for InMemorySource {
    fn modified(&self, _path: &str) -> Result<u64, Error> {
        Ok(0)
    }

    fn load(&self, path: &str) -> Result<Vec<u8>, Error> {
        let path = path.to_string();
        self.0.get(&path).cloned().ok_or_else(|| {
            format_err!(
                "The `{}` asset is not registered in the `InMemorySource` asset source",
                path
            )
        })
    }
}
