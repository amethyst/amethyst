use crate::{
    loader::{AssetType, AssetTypeStorage, DefaultLoader},
    prefab::{ComponentRegistry, Prefab, RawPrefab},
    processor::LoadNotifier,
    storage::AssetStorage,
    LoadHandle,
};
use amethyst_core::{
    dispatcher::System,
    ecs::{systems::ParallelRunnable, SystemBuilder},
};
use amethyst_error::Error as AmethystError;
use atelier_assets::loader::{storage::AssetLoadOp, AssetTypeId};
use crossbeam_queue::SegQueue;
use std::{
    collections::HashMap,
    error::Error as StdError,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};
use type_uuid::TypeUuid;

/// Creates an `AssetType` to be stored in the `AssetType` `inventory`.
///
/// This function is not intended to be called be directly. Use the `register_asset_type!` macro
/// macro instead.
pub fn create_asset_type() -> AssetType {
    log::debug!("Creating asset type: {:x?}", RawPrefab::UUID);
    AssetType {
        data_uuid: AssetTypeId(RawPrefab::UUID),
        asset_uuid: AssetTypeId(RawPrefab::UUID),
        create_storage: |res, indirection_table| {
            res.get_or_insert_with(|| AssetStorage::<RawPrefab>::new(indirection_table.clone()));
            res.get_or_insert_with(|| AssetStorage::<Prefab>::new(indirection_table.clone()));
            res.get_or_insert_with(PrefabProcessingQueue::default);
        },
        register_system: |builder| {
            builder.add_system(Box::new(PrefabAssetProcessor::default()));
        },
        with_storage: |res, func| {
            func(&mut (
                res.get::<PrefabProcessingQueue>()
                    .expect("Could not get ProcessingQueue")
                    .deref(),
                res.get_mut::<AssetStorage<RawPrefab>>()
                    .expect("Could not get_mut AssetStorage")
                    .deref_mut(),
            ))
        },
    }
}

impl AssetTypeStorage for (&PrefabProcessingQueue, &mut AssetStorage<RawPrefab>) {
    fn update_asset(
        &self,
        handle: LoadHandle,
        data: std::vec::Vec<u8>,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn StdError + Send>> {
        log::debug!("AssetTypeStorage update_asset");
        match bincode::deserialize::<RawPrefab>(data.as_ref()) {
            Err(err) => {
                log::debug!("Error in AssetTypeStorage deserialize");
                let e = AmethystError::from_string(format!("{}", err));
                load_op.error(err);
                Err(e.into_error())
            }
            Ok(asset) => {
                log::debug!("Ok in AssetTypeStorag deserialize");
                self.0.enqueue(handle, asset, load_op, version);
                Ok(())
            }
        }
    }
    fn commit_asset_version(&mut self, handle: LoadHandle, version: u32) {
        self.1.commit_asset(handle, version);
    }
    fn free(&mut self, handle: LoadHandle, version: u32) {
        self.1.remove_asset(handle, version);
    }
}

/// Represents asset data processed by `atelier-assets` that needs to be loaded by Amethyst.
pub(crate) struct Processed<T> {
    data: Result<T, AmethystError>,
    handle: LoadHandle,
    load_notifier: LoadNotifier,
    version: u32,
    commit: bool,
}

/// Returned by processor systems, describes the loading state of the asset.
pub enum ProcessingState<D, A> {
    /// Asset is not fully loaded yet, need to wait longer
    Loading(D),
    /// Asset have finished loading, can now be inserted into storage and tracker notified
    Loaded(A),
}

/// Queue of processed asset data, to be loaded by Amethyst.
///
/// # Type Parameters
///
/// `T`: Asset data type.
pub struct PrefabProcessingQueue {
    pub(crate) processed: Arc<SegQueue<Processed<RawPrefab>>>,
    requeue: Mutex<Vec<Processed<RawPrefab>>>,
}

impl Default for PrefabProcessingQueue {
    fn default() -> Self {
        Self {
            processed: Arc::new(SegQueue::new()),
            requeue: Mutex::new(Vec::new()),
        }
    }
}

impl PrefabProcessingQueue {
    /// Enqueue asset data for processing
    pub(crate) fn enqueue(
        &self,
        handle: LoadHandle,
        data: RawPrefab,
        asset_load_op: AssetLoadOp,
        version: u32,
    ) {
        self.enqueue_processed(
            Ok(data),
            handle,
            LoadNotifier::new(handle, Some(asset_load_op), None),
            version,
            false,
        );
    }

    pub(crate) fn enqueue_processed(
        &self,
        data: Result<RawPrefab, AmethystError>,
        handle: LoadHandle,
        load_notifier: LoadNotifier,
        version: u32,
        commit: bool,
    ) {
        self.processed.push(Processed {
            data,
            handle,
            load_notifier,
            version,
            commit,
        })
    }

    // pub(crate) fn enqueue_from_data(
    //     &self,
    //     handle: LoadHandle,
    //     data: T,
    //     tracker: Box<dyn Tracker>,
    //     version: u32,
    // ) {
    //     self.enqueue_processed(
    //         Ok(data),
    //         handle,
    //         LoadNotifier::new(handle, None, Some(tracker)),
    //         version,
    //         true,
    //     );
    // }

    /// Process asset data into assets
    pub fn process(
        &mut self,
        raw_storage: &mut AssetStorage<RawPrefab>,
        storage: &mut AssetStorage<Prefab>,
        component_registry: &ComponentRegistry,
        loader: &DefaultLoader,
    ) {
        {
            let requeue = self
                .requeue
                .get_mut()
                .expect("The mutex of `requeue` in `AssetStorage` was poisoned");
            while let Ok(processed) = self.processed.pop() {
                let Processed {
                    data,
                    handle,
                    load_notifier,
                    version,
                    commit,
                } = processed;

                let asset = match data.and_then(|RawPrefab { raw_prefab }| {
                    let prefab_cook_order = vec![raw_prefab.prefab_id()];
                    let mut prefab_lookup = HashMap::new();
                    prefab_lookup.insert(raw_prefab.prefab_id(), &raw_prefab);

                    let prefab = legion_prefab::cook_prefab(
                        component_registry.components(),
                        component_registry.components_by_uuid(),
                        prefab_cook_order.as_slice(),
                        &prefab_lookup,
                    );

                    Ok(ProcessingState::Loaded(RawPrefab { raw_prefab }))
                }) {
                    Ok(ProcessingState::Loaded(x)) => {
                        log::debug!(
                            "Asset (handle id: {:?}) has been loaded successfully",
                            handle,
                        );
                        load_notifier.complete();
                        x
                    }
                    Ok(ProcessingState::Loading(x)) => {
                        requeue.push(Processed {
                            data: Ok(x),
                            handle,
                            load_notifier,
                            version,
                            commit,
                        });
                        continue;
                    }
                    Err(e) => {
                        load_notifier.error(e);
                        continue;
                    }
                };
                raw_storage.update_asset(handle, asset, version);
                if commit {
                    raw_storage.commit_asset(handle, version);
                }
            }

            for p in requeue.drain(..) {
                self.processed.push(p);
            }
        }
    }
}
#[derive(Default)]
struct PrefabAssetProcessor;

impl System<'static> for PrefabAssetProcessor {
    fn build(&'static mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("PrefabAssetProcessorSystem")
                .read_resource::<ComponentRegistry>()
                .write_resource::<PrefabProcessingQueue>()
                .write_resource::<AssetStorage<RawPrefab>>()
                .write_resource::<AssetStorage<Prefab>>()
                .write_resource::<DefaultLoader>()
                .build(
                    move |_,
                          _,
                          (
                        component_registry,
                        processing_queue,
                        raw_prefab_storage,
                        prefab_storage,
                        loader,
                    ),
                          _| {
                        prefab_asset_processor(
                            component_registry,
                            processing_queue,
                            raw_prefab_storage,
                            prefab_storage,
                            loader,
                        );
                    },
                ),
        )
    }
}

fn prefab_asset_processor(
    component_registry: &ComponentRegistry,
    processing_queue: &mut PrefabProcessingQueue,
    raw_prefab_storage: &mut AssetStorage<RawPrefab>,
    prefab_storage: &mut AssetStorage<Prefab>,
    loader: &mut DefaultLoader,
) {
    #[cfg(feature = "profiler")]
    profile_scope!("prefab_asset_processor");

    processing_queue.process(
        raw_prefab_storage,
        prefab_storage,
        component_registry,
        loader,
    );
    prefab_storage.process_custom_drop(|_| {});
    raw_prefab_storage.process_custom_drop(|_| {});
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prefab::{ComponentRegistryBuilder, Prefab, RawPrefabMapping, RootPrefabs};
    use crate::{processor::LoadNotifier, Handle, LoadHandle};
    use amethyst_core::ecs::World;
    use atelier_assets::loader::{
        crossbeam_channel::{unbounded, Sender},
        handle::{AssetHandle, RefOp},
        storage::{AtomicHandleAllocator, HandleAllocator},
    };
    use hamcrest2::prelude::*;
    use std::sync::Arc;

    struct Fixture {
        root_prefabs: RootPrefabs,
        loader: DefaultLoader,
        processing_queue: PrefabProcessingQueue,
        prefab_storage: AssetStorage<Prefab>,
        raw_prefab_storage: AssetStorage<RawPrefab>,
        component_registry: ComponentRegistry,
        handle_maker: HandleMaker,
    }

    impl Fixture {
        fn setup() -> Self {
            let root_prefabs = RootPrefabs::default();
            let loader = DefaultLoader::new(root_prefabs.clone());
            let processing_queue = PrefabProcessingQueue::default();
            let prefab_storage = AssetStorage::<Prefab>::new(loader.indirection_table.clone());
            let raw_prefab_storage =
                AssetStorage::<RawPrefab>::new(loader.indirection_table.clone());
            let component_registry = ComponentRegistryBuilder::default()
                .auto_register_components()
                .build();
            let handle_allocator = Arc::new(AtomicHandleAllocator::default());
            let (ref_sender, ref_receiver) = unbounded();
            let handle_maker = HandleMaker::new(handle_allocator, ref_sender);
            Self {
                root_prefabs,
                loader,
                processing_queue,
                prefab_storage,
                raw_prefab_storage,
                component_registry,
                handle_maker,
            }
        }
    }

    struct HandleMaker {
        handle_allocator: Arc<AtomicHandleAllocator>,
        ref_sender: Sender<RefOp>,
    }

    impl HandleMaker {
        fn new(handle_allocator: Arc<AtomicHandleAllocator>, ref_sender: Sender<RefOp>) -> Self {
            Self {
                handle_allocator,
                ref_sender,
            }
        }
        fn make_handle<T>(&self) -> Handle<T> {
            let load_handle = self.handle_allocator.alloc();
            Handle::<T>::new(self.ref_sender.clone(), load_handle)
        }
    }

    #[test]
    fn test() {
        let Fixture {
            root_prefabs,
            mut loader,
            mut processing_queue,
            mut prefab_storage,
            mut raw_prefab_storage,
            component_registry,
            handle_maker,
        } = Fixture::setup();

        let raw_prefab_handle = handle_maker.make_handle();
        let prefab_handle = handle_maker.make_handle::<Prefab>();

        root_prefabs.insert(
            raw_prefab_handle.load_handle(),
            RawPrefabMapping {
                raw_prefab_handle: raw_prefab_handle.clone(),
                prefab_load_handle: prefab_handle.load_handle(),
            },
        );

        let prefab_world = World::default();
        let raw_prefab = RawPrefab {
            raw_prefab: legion_prefab::Prefab::new(prefab_world),
        };
        let version = 0;

        let load_notifier = LoadNotifier::new(raw_prefab_handle.load_handle(), None, None);
        processing_queue.enqueue_processed(
            Ok(raw_prefab),
            raw_prefab_handle.load_handle(),
            load_notifier,
            version,
            false,
        );

        prefab_asset_processor(
            &component_registry,
            &mut processing_queue,
            &mut raw_prefab_storage,
            &mut prefab_storage,
            &mut loader,
        );

        raw_prefab_storage.commit_asset(raw_prefab_handle.load_handle(), version);

        let asset = prefab_storage.get(&raw_prefab_handle);
        assert!(asset.is_some());
    }
}
