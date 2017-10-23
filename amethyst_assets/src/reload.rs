//! Defines the `Reload` trait.

use std::sync::Arc;
use std::time::Instant;

use {Asset, BoxedErr, Format, FormatValue, Source};

/// An ECS resource which allows to configure hot reloading.
///
/// ## Examples
///
/// ```
/// # extern crate amethyst_assets;
/// # extern crate specs;
/// #
/// # use amethyst_assets::HotReloadStrategy;
/// # use specs::World;
/// #
/// # fn main() {
/// let mut world = World::new();
/// // Assets will be reloaded every two seconds (in case they changed)
/// world.add_resource(HotReloadStrategy::every(2));
/// # }
/// ```
pub struct HotReloadStrategy {
    inner: HotReloadStrategyInner,
}

impl HotReloadStrategy {
    /// Causes hot reloads every `n` seconds.
    pub fn every(n: u8) -> Self {
        HotReloadStrategy {
            inner: HotReloadStrategyInner::Every {
                interval: n,
                start: Instant::now(),
            },
        }
    }

    /// No periodical hot-reloading is performed.
    /// Instead, you can `trigger` it to check for changed assets
    /// in one specific frame.
    pub fn never() -> Self {
        HotReloadStrategy {
            inner: HotReloadStrategyInner::Never,
        }
    }

    /// The frame after calling this, all changed assets will be reloaded.
    pub fn trigger(&mut self) {
        let counter = match self.inner {
            HotReloadStrategyInner::Trigger { counter } => counter.checked_add(1).unwrap_or(0),
            _ => 0,
        };

        self.inner = HotReloadStrategyInner::Trigger { counter };
    }

    /// Crate-internal method to check if reload is necessary.
    /// `reload_counter` is a per-storage value which is only used
    /// for and by this method.
    pub(crate) fn needs_reload(&self, reload_counter: &mut u64) -> bool {
        match self.inner {
            HotReloadStrategyInner::Every { interval, start } => {
                let now = Instant::now().duration_since(start).as_secs();
                if now - *reload_counter >= interval as u64 {
                    *reload_counter = now;

                    true
                } else {
                    false
                }
            }
            HotReloadStrategyInner::Trigger { counter } => {
                let counter = counter as u64;
                if counter != *reload_counter {
                    *reload_counter = counter;

                    true
                } else {
                    false
                }
            }
            HotReloadStrategyInner::Never => false,
        }
    }
}

enum HotReloadStrategyInner {
    Every { interval: u8, start: Instant },
    Trigger { counter: u8 },
    Never,
}

/// The `Reload` trait provides a method which checks if an asset needs to be reloaded.
pub trait Reload<A: Asset>: ReloadClone<A> + Send + Sync + 'static {
    /// Checks if a reload is necessary.
    fn needs_reload(&self) -> bool;
    /// Returns the asset name.
    fn name(&self) -> String;
    /// Returns the format name.
    fn format(&self) -> &'static str;
    /// Reloads the asset.
    fn reload(self: Box<Self>) -> Result<FormatValue<A>, BoxedErr>;
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

    fn reload(self: Box<Self>) -> Result<FormatValue<A>, BoxedErr> {
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

    fn format(&self) -> &'static str {
        F::NAME
    }
}
