use amethyst::{assets::ProgressCounter, ecs::WorldExt, State, StateData, Trans};
use derive_new::new;

use crate::GameUpdate;

/// Reads a `ProgressCounter` resource and waits for it to be `complete()`.
#[derive(Debug, new)]
pub struct WaitForLoad;

impl<T, E> State<T, E> for WaitForLoad
where
    T: GameUpdate,
    E: Send + Sync + 'static,
{
    fn update(&mut self, data: StateData<'_, T>) -> Trans<T, E> {
        data.data.update(&data.world);

        let progress_counter = data.world.read_resource::<ProgressCounter>();
        if !progress_counter.is_complete() {
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
        ecs::{storage::VecStorage, WorldExt},
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
                in_memory_source.insert(String::from("file.ron"), "(val: 123)".as_bytes().to_vec());

                let mut loader = world.write_resource::<Loader>();
                loader.add_source(IN_MEMORY_SOURCE_ID, in_memory_source);
            })
            .with_effect(|world| {
                let mut progress_counter = ProgressCounter::new();
                let test_asset_handle = {
                    let loader = world.read_resource::<Loader>();
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

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    pub struct TestAsset {
        val: u32,
    }

    impl Asset for TestAsset {
        type Data = Self;
        type HandleStorage = VecStorage<Handle<Self>>;

        const NAME: &'static str = concat!(module_path!(), "::", stringify!(TestAsset));
    }

    impl From<TestAsset> for Result<ProcessingState<TestAsset>, Error> {
        fn from(asset_data: TestAsset) -> Result<ProcessingState<TestAsset>, Error> {
            Ok(ProcessingState::Loaded(asset_data))
        }
    }
}
