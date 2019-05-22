//! Defines the `Reload` trait.

use std::{sync::Arc, time::Instant};

use amethyst_core::{
    ecs::prelude::{DispatcherBuilder, Read, Resources, System, Write},
    SystemBundle, Time,
};
use amethyst_error::Error;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{Format, FormatValue, Loader, Source};

/// This bundle activates hot reload for the `Loader`,
/// adds a `HotReloadStrategy` and the `HotReloadSystem`.
#[derive(Default)]
pub struct HotReloadBundle {
    strategy: HotReloadStrategy,
}

impl HotReloadBundle {
    /// Creates a new bundle.
    pub fn new(strategy: HotReloadStrategy) -> Self {
        HotReloadBundle { strategy }
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for HotReloadBundle {
    fn build(self, dispatcher: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        dispatcher.add(HotReloadSystem::new(self.strategy), "hot_reload", &[]);
        Ok(())
    }
}

/// An ECS resource which allows to configure hot reloading.
///
/// ## Examples
///
/// ```
/// # use amethyst_assets::HotReloadStrategy;
/// # use amethyst_core::ecs::prelude::World;
/// #
/// # fn main() {
/// let mut world = World::new();
/// // Assets will be reloaded every two seconds (in case they changed)
/// world.add_resource(HotReloadStrategy::every(2));
/// # }
/// ```
#[derive(Clone)]
pub struct HotReloadStrategy {
    inner: HotReloadStrategyInner,
}

impl HotReloadStrategy {
    /// Causes hot reloads every `n` seconds.
    pub fn every(n: u8) -> Self {
        use std::u64::MAX;

        HotReloadStrategy {
            inner: HotReloadStrategyInner::Every {
                interval: n,
                last: Instant::now(),
                frame_number: MAX,
            },
        }
    }

    /// This allows to use `trigger` for hot reloading.
    pub fn when_triggered() -> Self {
        use std::u64::MAX;

        HotReloadStrategy {
            inner: HotReloadStrategyInner::Trigger {
                triggered: false,
                frame_number: MAX,
            },
        }
    }

    /// Never do any hot-reloading.
    pub fn never() -> Self {
        HotReloadStrategy {
            inner: HotReloadStrategyInner::Never,
        }
    }

    /// The frame after calling this, all changed assets will be reloaded.
    /// Doesn't do anything if the strategy wasn't created with `when_triggered`.
    pub fn trigger(&mut self) {
        if let HotReloadStrategyInner::Trigger {
            ref mut triggered, ..
        } = self.inner
        {
            *triggered = true;
        }
    }

    /// Crate-internal method to check if reload is necessary.
    /// `reload_counter` is a per-storage value which is only used
    /// for and by this method.
    pub(crate) fn needs_reload(&self, current_frame: u64) -> bool {
        match self.inner {
            HotReloadStrategyInner::Every { frame_number, .. } => frame_number == current_frame,
            HotReloadStrategyInner::Trigger { frame_number, .. } => frame_number == current_frame,
            HotReloadStrategyInner::Never => false,
        }
    }
}

impl Default for HotReloadStrategy {
    fn default() -> Self {
        HotReloadStrategy::every(1)
    }
}

#[derive(Clone)]
enum HotReloadStrategyInner {
    Every {
        interval: u8,
        last: Instant,
        frame_number: u64,
    },
    Trigger {
        triggered: bool,
        frame_number: u64,
    },
    Never,
}

/// System for updating `HotReloadStrategy`.
pub struct HotReloadSystem {
    initial_strategy: HotReloadStrategy,
}

impl HotReloadSystem {
    /// Create a new reload system
    pub fn new(strategy: HotReloadStrategy) -> Self {
        HotReloadSystem {
            initial_strategy: strategy,
        }
    }
}

impl<'a> System<'a> for HotReloadSystem {
    type SystemData = (Read<'a, Time>, Write<'a, HotReloadStrategy>);

    fn run(&mut self, (time, mut strategy): Self::SystemData) {
        match strategy.inner {
            HotReloadStrategyInner::Trigger {
                ref mut triggered,
                ref mut frame_number,
            } => {
                if *triggered {
                    *frame_number = time.frame_number() + 1;
                }
                *triggered = false;
            }
            HotReloadStrategyInner::Every {
                interval,
                ref mut last,
                ref mut frame_number,
            } => {
                if last.elapsed().as_secs() > u64::from(interval) {
                    *frame_number = time.frame_number() + 1;
                    *last = Instant::now();
                }
            }
            HotReloadStrategyInner::Never => {}
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::ecs::prelude::SystemData;
        Self::SystemData::setup(res);
        res.insert(self.initial_strategy.clone());
        res.fetch_mut::<Loader>().set_hot_reload(true);
    }
}

/// The `Reload` trait provides a method which checks if an asset needs to be reloaded.
pub trait Reload<D>: ReloadClone<D> + Send + Sync + 'static {
    /// Checks if a reload is necessary.
    fn needs_reload(&self) -> bool;
    /// Returns the asset name.
    fn name(&self) -> String;
    /// Returns the format name.
    fn format(&self) -> &'static str;
    /// Reloads the asset.
    fn reload(self: Box<Self>) -> Result<FormatValue<D>, Error>;
}

pub trait ReloadClone<D> {
    fn cloned(&self) -> Box<dyn Reload<D>>;
}

impl<D: 'static, T> ReloadClone<D> for T
where
    T: Clone + Reload<D>,
{
    fn cloned(&self) -> Box<dyn Reload<D>> {
        Box::new(self.clone())
    }
}

impl<D: 'static> Clone for Box<dyn Reload<D>> {
    fn clone(&self) -> Self {
        self.cloned()
    }
}

/// An implementation of `Reload` which just stores the modification time
/// and the path of the file.
pub struct SingleFile<D> {
    format: Box<dyn Format<D>>,
    modified: u64,
    path: String,
    source: Arc<dyn Source>,
}

impl<D: 'static> SingleFile<D> {
    /// Creates a new `SingleFile` reload object.
    pub fn new(
        format: Box<dyn Format<D>>,
        modified: u64,
        path: String,
        source: Arc<dyn Source>,
    ) -> Self {
        SingleFile {
            format,
            modified,
            path,
            source,
        }
    }
}

impl<D: 'static> Clone for SingleFile<D> {
    fn clone(&self) -> Self {
        SingleFile {
            format: self.format.clone(),
            modified: self.modified,
            path: self.path.clone(),
            source: self.source.clone(),
        }
    }
}

impl<D: 'static> Reload<D> for SingleFile<D> {
    fn needs_reload(&self) -> bool {
        self.modified != 0 && (self.source.modified(&self.path).unwrap_or(0) > self.modified)
    }

    fn name(&self) -> String {
        self.path.clone()
    }

    fn format(&self) -> &'static str {
        self.format.name()
    }

    fn reload(self: Box<Self>) -> Result<FormatValue<D>, Error> {
        #[cfg(feature = "profiler")]
        profile_scope!("reload_single_file");

        let this: SingleFile<D> = *self;
        let SingleFile {
            format,
            path,
            source,
            ..
        } = this;

        format.import(path, source, Some(objekt::clone(&format)))
    }
}
