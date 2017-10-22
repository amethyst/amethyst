//! Defines the `Reload` trait.

use std::sync::Arc;

use {Asset, BoxedErr, Format, Source};

/// The `Reload` trait provides a method which checks if an asset needs to be reloaded.
pub trait Reload<A: Asset>: ReloadClone<A> + Send + Sync + 'static {
    /// Checks if a reload is necessary.
    fn needs_reload(&self) -> bool;
    /// Returns the asset name.
    fn name(&self) -> String;
    /// Returns the format name.
    fn format(&self) -> String;
    /// Reloads the asset.
    fn reload(self: Box<Self>) -> Result<(A::Data, Option<Box<Reload<A>>>), BoxedErr>;
}

pub trait ReloadClone<A> {
    fn cloned(&self) -> Box<Reload<A>>;
}

impl<A, T> ReloadClone<A> for T
where
    A: Asset,
    T: Clone + Reload<A>,
{
    fn cloned(&self) -> Box<Reload<A>> {
        Box::new(self.clone())
    }
}

impl<A: Asset> Clone for Box<Reload<A>> {
    fn clone(&self) -> Self {
        self.cloned()
    }
}

/// An implementation of `Reload` which just stores the modification time
/// and the path of the file.
pub struct SingleFile<A: Asset, F: Format<A>> {
    format: F,
    modified: u64,
    options: F::Options,
    path: String,
    source: Arc<Source>,
}

impl<A: Asset, F: Format<A>> SingleFile<A, F> {
    /// Creates a new `SingleFile` reload object.
    pub fn new(
        format: F,
        modified: u64,
        options: F::Options,
        path: String,
        source: Arc<Source>,
    ) -> Self {
        SingleFile {
            format,
            modified,
            options,
            path,
            source,
        }
    }
}

impl<A, F> Clone for SingleFile<A, F>
where
    A: Asset,
    F: Clone + Format<A>,
    F::Options: Clone,
{
    fn clone(&self) -> Self {
        SingleFile {
            format: self.format.clone(),
            modified: self.modified,
            options: self.options.clone(),
            path: self.path.clone(),
            source: self.source.clone(),
        }
    }
}

impl<A, F> Reload<A> for SingleFile<A, F>
where
    A: Asset,
    F: Clone + Format<A> + Sync,
    <F as Format<A>>::Options: Clone + Sync,
{
    fn needs_reload(&self) -> bool {
        self.modified != 0 && (self.source.modified(&self.path).unwrap_or(0) > self.modified)
    }

    fn reload(self: Box<Self>) -> Result<(A::Data, Option<Box<Reload<A>>>), BoxedErr> {
        let this: SingleFile<_, _> = *self;
        let SingleFile {
            format,
            path,
            source,
            options,
            ..
        } = this;

        format.import(path, source, options, true)
    }

    fn name(&self) -> String {
        self.path.clone()
    }

    fn format(&self) -> String {
        F::NAME.to_owned()
    }
}
