use std::time::Duration;

use amethyst::{assets::ProgressCounter, core::Stopwatch, ecs::World, State, StateData, Trans};
use derivative::Derivative;
use log::warn;

use crate::GameUpdate;

/// Time limit before outputting a warning message.
const LOADING_TIME_LIMIT: Duration = Duration::from_secs(10);

/// Reads a `ProgressCounter` resource and waits for it to be `complete()`.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct WaitForLoad {
    /// Tracks how long the `WaitForLoad` state has run.
    ///
    /// Used to output a warning if loading takes too long.
    stopwatch: Stopwatch,
    /// Function to determine loading is complete.
    #[derivative(Debug = "ignore")]
    fn_complete: fn(&World) -> bool,
}

impl WaitForLoad {
    /// Returns a `WaitForLoad` that assumes a `ProgressCounter` resource exists in the `World`.
    pub fn new() -> Self {
        WaitForLoad {
            fn_complete: |world| world.read_resource::<ProgressCounter>().is_complete(),
            stopwatch: Stopwatch::new(),
        }
    }

    /// Returns a `WaitForLoad` with a custom completion check.
    ///
    /// # Parameters
    ///
    /// * `fn_complete`: Function to determine loading is complete.
    pub fn new_with_fn(fn_complete: fn(&World) -> bool) -> Self {
        WaitForLoad {
            fn_complete,
            stopwatch: Stopwatch::new(),
        }
    }
}

impl<T, E> State<T, E> for WaitForLoad
where
    T: GameUpdate,
    E: Send + Sync + 'static,
{
    fn on_start(&mut self, _data: StateData<'_, T>) {
        self.stopwatch.start();
    }

    fn on_resume(&mut self, _data: StateData<'_, T>) {
        self.stopwatch.restart();
    }

    fn update(&mut self, data: StateData<'_, T>) -> Trans<T, E> {
        data.data.update(&data.world);

        if !(self.fn_complete)(&data.world) {
            if let Stopwatch::Started(..) = &self.stopwatch {
                let elapsed = self.stopwatch.elapsed();
                if elapsed > LOADING_TIME_LIMIT {
                    self.stopwatch.stop();

                    warn!(
                        "Loading has not completed in 10 seconds, please ensure that you have \
                         registered the relevant `Processor::<A>`s in the dispatcher.",
                    );
                }
            }

            Trans::None
        } else {
            Trans::Pop
        }
    }
}

#[cfg(test)]
mod tests {
    use amethyst::{
        assets::{
            Asset, AssetStorage, Handle, Loader, ProcessingState, Processor, ProgressCounter,
            RonFormat,
        },
        Error,
    };
    use serde::{Deserialize, Serialize};

    use super::WaitForLoad;
    use crate::{AmethystApplication, InMemorySource, IN_MEMORY_SOURCE_ID};

    #[test]
    fn pops_when_progress_counter_is_complete() -> Result<(), Error> {
        AmethystApplication::blank()
            .with_system(Processor::<TestAsset>::new(), "test_asset_processor", &[])
            .with_effect(|world| {
                let mut in_memory_source = InMemorySource::new();
                in_memory_source.insert(String::from("file.ron"), b"(val: 123)".to_vec());

                let mut loader = world.write_resource::<DefaultLoader>();
                loader.add_source(IN_MEMORY_SOURCE_ID, in_memory_source);
            })
            .with_effect(|world| {
                let mut progress_counter = ProgressCounter::new();
                let test_asset_handle = {
                    let loader = data.resources.get::<DefaultLoader>().unwrap();
                    loader.load_from(
                        "file.ron",
                        RonFormat,
                        IN_MEMORY_SOURCE_ID,
                        &mut progress_counter,
                        &world.read_resource::<AssetStorage<TestAsset>>(),
                    )
                };

                world.insert(test_asset_handle);
                world.insert(progress_counter);
            })
            .with_state(WaitForLoad::new)
            .with_assertion(|world| {
                let test_asset_handle = world.read_resource::<Handle<TestAsset>>();
                let test_assets = world.read_resource::<AssetStorage<TestAsset>>();
                let test_asset = test_assets
                    .get(&test_asset_handle)
                    .expect("Expected `TestAsset` to be loaded.");

                assert_eq!(&TestAsset { val: 123 }, test_asset);
            })
            .run()
    }

    #[test]
    fn uses_custom_completion_function() -> Result<(), Error> {
        AmethystApplication::blank()
            .with_system(Processor::<TestAsset>::new(), "test_asset_processor", &[])
            .with_effect(|world| {
                let mut in_memory_source = InMemorySource::new();
                in_memory_source.insert(String::from("file.ron"), b"(val: 123)".to_vec());

                let mut loader = world.write_resource::<DefaultLoader>();
                loader.add_source(IN_MEMORY_SOURCE_ID, in_memory_source);
            })
            .with_effect(|world| {
                let mut progress_counter = ProgressCounter::new();
                let test_asset_handle = {
                    let loader = data.resources.get::<DefaultLoader>().unwrap();
                    loader.load_from(
                        "file.ron",
                        RonFormat,
                        IN_MEMORY_SOURCE_ID,
                        &mut progress_counter,
                        &world.read_resource::<AssetStorage<TestAsset>>(),
                    )
                };

                world.insert(test_asset_handle);
                world.insert(vec![progress_counter]);
            })
            .with_state(|| {
                WaitForLoad::new_with_fn(|world| {
                    world
                        .read_resource::<Vec<ProgressCounter>>()
                        .first()
                        .expect("Expected `Vec<ProgressCounter>` with one element.")
                        .is_complete()
                })
            })
            .with_assertion(|world| {
                let test_asset_handle = world.read_resource::<Handle<TestAsset>>();
                let test_assets = world.read_resource::<AssetStorage<TestAsset>>();
                let test_asset = test_assets
                    .get(&test_asset_handle)
                    .expect("Expected `TestAsset` to be loaded.");

                assert_eq!(&TestAsset { val: 123 }, test_asset);
            })
            .run()
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    pub struct TestAsset {
        val: u32,
    }

    impl Asset for TestAsset {
        type Data = Self;

        const NAME: &'static str = concat!(module_path!(), "::", stringify!(TestAsset));
    }

    impl From<TestAsset> for Result<ProcessingState<TestAsset>, Error> {
        fn from(asset_data: TestAsset) -> Result<ProcessingState<TestAsset>, Error> {
            Ok(ProcessingState::Loaded(asset_data))
        }
    }
}
