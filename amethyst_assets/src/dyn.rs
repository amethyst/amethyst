#![allow(unused)] // TODO: remove

use std::fmt;
use std::marker::PhantomData;

use fnv::FnvHashMap;
use ron::de::{from_str, Error as RonError};
use serde::{Deserialize, Deserializer};
use serde::de::DeserializeSeed;

use {Asset, AssetStorage, ErrorKind, Format, Handle, Loader, Progress, Result, ResultExt};

/// An assets deserializer, allowing to load assets from RON files.
pub struct AssetsDeserializer<'a, A: Asset> {
    collection: FormatCollection<A>,
    loader: &'a Loader,
    progress: DynProgress,
    storage: &'a AssetStorage<A>,
}

impl<'a, A> AssetsDeserializer<'a, A>
where
    A: Asset,
{
    /// Creates a new instance of `AssetsDeserializer`.
    pub fn new(loader: &'a Loader, storage: &'a AssetStorage<A>) -> Self {
        AssetsDeserializer {
            collection: FormatCollection::new(),
            loader,
            progress: DynProgress::new(),
            storage,
        }
    }

    /// Adds a format with `name`. Only added formats can be specified
    /// in the RON file.
    pub fn with_format<N, T>(&mut self, name: N, format: T) -> &mut Self
    where
        N: Into<String>,
        T: Format<A> + Clone,
        T::Options: Default,
    {
        self.collection.add(name.into(), format);

        self
    }

    /// Loads a mapping of `String` to asset handles from a given `source`.
    pub fn load_from(&mut self, name: &str, source: &str) -> Result<FnvHashMap<String, Handle<A>>> {
        let bytes = self.loader.source(source).load(name)?;
        let s = String::from_utf8(bytes)?;
        // TODO: optimize
        from_str::<FnvHashMap<String, Descriptor>>(s.as_ref())
            .chain_err(|| "Failed to deserialize assets file")?
            .into_iter()
            .map(|(key, value)| {
                Ok((
                    key,
                    self.collection.load(
                        self.loader,
                        value.name,
                        &value.format,
                        value.source,
                        &mut self.progress,
                        self.storage,
                    )?,
                ))
            })
            .collect()
    }
}

#[derive(Deserialize)]
struct Descriptor {
    name: String,
    format: String,
    #[serde(default = "String::new")]
    source: String,
}

/// A dynamic format, stored in the `FormatCollection::map` field.
/// There's only one implementation of this trait,
/// for `(F, D) where D: OptionsDeserializer<..>, F: Format<..>,`.
trait DynFormat<A: Asset> {
    /// Only returns `Err` if the deserialization of `options` failed.
    fn visit_loader(
        &self,
        loader: &Loader,
        name: String,
        source: String,
        progress: &mut DynProgress,
        storage: &AssetStorage<A>,
    ) -> Result<Handle<A>>;
}

impl<A, F> DynFormat<A> for F
where
    A: Asset,
    F: Format<A> + Clone,
    F::Options: Default,
{
    fn visit_loader(
        &self,
        loader: &Loader,
        name: String,
        source: String,
        progress: &mut DynProgress,
        storage: &AssetStorage<A>,
    ) -> Result<Handle<A>> {
        let format = self.clone();
        let options = Default::default(); // TODO: allow deserializing options
        let handle = loader.load_from(name, format, options, &source, progress, storage);

        // TODO: this technically does not need to return `Result`,
        // TODO: but it will for deserialization.

        Ok(handle)
    }
}

/// A collection of formats, allowing to choose a format
/// dynamically which is useful for deserialization.
///
/// The type parameter `A` is the asset type.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
struct FormatCollection<A: Asset> {
    map: FnvHashMap<String, Box<DynFormat<A>>>,
}

impl<A: Asset> FormatCollection<A> {
    /// Creates a new, empty format collection.
    fn new() -> Self {
        Default::default()
    }

    /// Add a format to the format collection.
    fn add<F>(&mut self, id: String, format: F)
    where
        F: Format<A> + Clone,
        F::Options: Default,
    {
        let dyn = Box::new(format);

        self.map.insert(id, dyn);
    }

    fn load(
        &self,
        loader: &Loader,
        name: String,
        format: &str,
        source: String,
        progress: &mut DynProgress,
        storage: &AssetStorage<A>,
    ) -> Result<Handle<A>> {
        self.map
            .get(format)
            .chain_err(|| ErrorKind::NoSuchFormat(format.to_owned()))?
            .visit_loader(loader, name, source, progress, storage)
    }
}

struct DynProgress {} // TODO: add fields

impl DynProgress {
    fn new() -> Self {
        DynProgress {}
    }
}

impl<'a> Progress for &'a mut DynProgress {
    type Tracker = ();

    fn add_assets(&mut self, _num: usize) {
        unimplemented!()
    }

    fn create_tracker(self) -> Self::Tracker {
        unimplemented!()
    }
}
