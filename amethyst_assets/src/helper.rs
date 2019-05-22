use std::{borrow::Borrow, hash::Hash};

use amethyst_core::ecs::{Read, ReadExpect};
use shred_derive::SystemData;

use crate::{Asset, AssetStorage, Format, Handle, Loader, Progress};

/// Helper type for loading assets
#[derive(SystemData)]
pub struct AssetLoaderSystemData<'a, A>
where
    A: Asset,
{
    loader: ReadExpect<'a, Loader>,
    storage: Read<'a, AssetStorage<A>>,
}

impl<'a, A> AssetLoaderSystemData<'a, A>
where
    A: Asset,
{
    /// Loads an asset with a given format from the default (directory) source.
    /// If you want to load from a custom source instead, use `load_from`.
    ///
    /// See `load_from` for more information.
    pub fn load<F, N, P>(&self, name: N, format: F, progress: P) -> Handle<A>
    where
        F: Format<A::Data>,
        N: Into<String>,
        P: Progress,
    {
        self.loader.load(name, format, progress, &*self.storage)
    }

    /// Loads an asset with a given id and format from a custom source.
    /// The actual work is done in a worker thread, thus this method immediately returns a handle.
    ///
    /// ## Parameters
    ///
    /// * `name`: this is just an identifier for the asset, most likely a file name e.g.
    ///   `"meshes/a.obj"`
    /// * `format`: A format struct which loads bytes from a `source` and produces `Asset::Data`
    ///   with them
    /// * `source`: An identifier for a source which has previously been added using `with_source`
    /// * `progress`: A tracker which will be notified of assets which have been imported
    pub fn load_from<F, N, P, S>(&self, name: N, format: F, source: &S, progress: P) -> Handle<A>
    where
        F: Format<A::Data>,
        N: Into<String>,
        P: Progress,
        S: AsRef<str> + Eq + Hash + ?Sized,
        String: Borrow<S>,
    {
        self.loader
            .load_from(name, format, source, progress, &*self.storage)
    }

    /// Load an asset from data and return a handle.
    pub fn load_from_data<P>(&self, data: A::Data, progress: P) -> Handle<A>
    where
        A: Asset,
        P: Progress,
    {
        self.loader.load_from_data(data, progress, &*self.storage)
    }
}
