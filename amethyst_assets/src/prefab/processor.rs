use crate::{
    loader::{AssetType, AssetTypeStorage, DefaultLoader, Loader},
    prefab::{ComponentRegistry, Prefab},
    processor::LoadNotifier,
    storage::AssetStorage,
    AssetHandle, LoadHandle, WeakHandle,
};
use amethyst_core::{
    dispatcher::System,
    ecs::{systems::ParallelRunnable, SystemBuilder},
};
use amethyst_error::Error as AmethystError;
use atelier_assets::{
    core::AssetUuid,
    loader::{storage::AssetLoadOp, AssetTypeId},
};
use crossbeam_queue::SegQueue;
use fnv::{FnvHashMap, FnvHashSet};
use prefab_format::PrefabUuid;
use std::{
    error::Error as StdError,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;
use type_uuid::TypeUuid;

pub fn create_prefab_asset_type() -> AssetType {
    log::debug!("Creating asset type: {:x?}", Prefab::UUID);
    AssetType {
        data_uuid: AssetTypeId(Prefab::UUID),
        asset_uuid: AssetTypeId(Prefab::UUID),
        create_storage: |res, indirection_table| {
            res.get_or_insert_with(|| AssetStorage::<Prefab>::new(indirection_table.clone()));
        },
        register_system: |builder| {
            builder.add_system(Box::new(PrefabAssetProcessor::default()));
        },
        with_storage: |res, func| {
            func(&mut (
                res.get::<PrefabProcessingQueue>()
                    .expect("Could not get ProcessingQueue")
                    .deref(),
                res.get_mut::<AssetStorage<Prefab>>()
                    .expect("Could not get_mut AssetStorage")
                    .deref_mut(),
            ))
        },
    }
}

inventory::submit!(create_prefab_asset_type());

impl AssetTypeStorage for (&PrefabProcessingQueue, &mut AssetStorage<Prefab>) {
    fn update_asset(
        &self,
        handle: LoadHandle,
        data: std::vec::Vec<u8>,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn StdError + Send>> {
        log::debug!("AssetTypeStorage update_asset");
        match bincode::deserialize::<Prefab>(data.as_ref()) {
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
        self.0.enqueue_changed(handle);
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
    pub(crate) processed: Arc<SegQueue<Processed<Prefab>>>,
    changed: SegQueue<LoadHandle>,
    requeue: Mutex<Vec<Processed<Prefab>>>,
}

impl Default for PrefabProcessingQueue {
    fn default() -> Self {
        Self {
            processed: Arc::new(SegQueue::new()),
            changed: SegQueue::new(),
            requeue: Mutex::new(Vec::new()),
        }
    }
}

impl PrefabProcessingQueue {
    /// Enqueue asset data for processing
    pub(crate) fn enqueue(
        &self,
        handle: LoadHandle,
        data: Prefab,
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
        data: Result<Prefab, AmethystError>,
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

    pub(crate) fn enqueue_changed(&self, handle: LoadHandle) {
        self.changed.push(handle);
    }

    fn cook_prefab(
        prefab: &Prefab,
        storage: &AssetStorage<Prefab>,
        component_registry: &ComponentRegistry,
    ) -> legion_prefab::CookedPrefab {
        // This will allow us to look up prefab references by AssetUuid
        let mut prefab_lookup = FnvHashMap::default();

        // This will hold the asset IDs sorted with dependencies first. This ensures that
        // prefab_lookup and entity_lookup are populated with all dependent prefabs/entities
        let mut prefab_cook_order: Vec<PrefabUuid> = vec![];
        let first_iter = prefab
            .dependencies
            .as_ref()
            .expect("dependencies have not been processed")
            .iter();
        let mut dependency_stack = vec![(prefab, first_iter)];
        loop {
            if let Some((cur_prefab, iter)) = dependency_stack.last_mut() {
                if let Some(next_handle) = iter.next() {
                    if let Some(next_prefab) = storage.get(next_handle) {
                        if prefab_lookup.contains_key(&next_prefab.raw_prefab.prefab_id()) {
                            continue;
                        }
                        let next_iter = next_prefab
                            .dependencies
                            .as_ref()
                            .expect("dependencies have not been processed")
                            .iter();
                        dependency_stack.push((next_prefab, next_iter));
                    } else {
                        log::error!("Missing prefab dependency");
                    }
                } else {
                    // No more dependencies, add cur_prefab to prefab_cook_order and
                    // pop the stack.
                    prefab_cook_order.push(cur_prefab.raw_prefab.prefab_id());
                    prefab_lookup.insert(cur_prefab.raw_prefab.prefab_id(), &cur_prefab.raw_prefab);
                    dependency_stack.pop();
                }
            } else {
                break;
            }
        }
        log::debug!("cook");
        log::debug!("prefab_cook_order: {:?}", prefab_cook_order);
        log::debug!("prefab_lookup: {:?}", prefab_lookup.keys());
        let cooked_prefab = legion_prefab::cook_prefab(
            component_registry.components(),
            component_registry.components_by_uuid(),
            prefab_cook_order.as_slice(),
            &prefab_lookup,
        );
        cooked_prefab
    }

    /// Process asset data into assets
    pub fn process(
        &mut self,
        storage: &mut AssetStorage<Prefab>,
        component_registry: &ComponentRegistry,
        loader: &impl Loader,
    ) {
        {
            {
                // cook prefabs with changed dependencies
                // FIXME: deal with cyclic and diamond dependencies correctly
                let mut visited = FnvHashSet::default();
                while let Ok(dependee) = self.changed.pop() {
                    let updates: Vec<(WeakHandle, legion_prefab::CookedPrefab)> = storage
                        .get_for_load_handle(dependee)
                        .iter()
                        .flat_map(|p| p.dependers.iter())
                        .flat_map(|weak_handle| {
                            storage
                                .get_asset_with_version(weak_handle)
                                .into_iter()
                                .map(move |(prefab, _)| (weak_handle, prefab))
                        })
                        .map(|(weak_handle, prefab)| {
                            let cooked_prefab =
                                Self::cook_prefab(prefab, storage, component_registry);
                            if visited.insert(weak_handle.load_handle()) {
                                self.changed.push(weak_handle.load_handle());
                            }
                            // FIXME: Add Clone to WeakHandle
                            (WeakHandle::new(weak_handle.load_handle()), cooked_prefab)
                        })
                        .collect();
                    use crate::storage::MutateAssetInStorage;
                    for (handle, cooked_prefab) in updates.into_iter() {
                        storage.mutate_asset_in_storage(&handle, move |asset| {
                            asset.prefab = Some(cooked_prefab);
                        });
                    }
                }
            }

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
                log::debug!("processing load_handle {:?}", handle);
                let mut prefab = match data.and_then(
                    |Prefab {
                         prefab,
                         raw_prefab,
                         mut dependencies,
                         dependers,
                     }| {
                        log::debug!("AssetUuid: {:?}", raw_prefab.prefab_id());
                        let deps = dependencies.get_or_insert_with(|| {
                            raw_prefab
                                .prefab_meta
                                .prefab_refs
                                .iter()
                                .map(|(other_prefab_id, _)| {
                                    loader.load_asset(AssetUuid(*other_prefab_id))
                                })
                                .collect()
                        });

                        if deps
                            .into_iter()
                            .all(|handle| storage.contains(handle.load_handle()))
                        {
                            Ok(ProcessingState::Loaded(Prefab {
                                prefab,
                                raw_prefab,
                                dependencies,
                                dependers,
                            }))
                        } else {
                            Ok(ProcessingState::Loading(Prefab {
                                prefab,
                                raw_prefab,
                                dependencies,
                                dependers,
                            }))
                        }
                    },
                ) {
                    Ok(ProcessingState::Loaded(raw)) => {
                        log::debug!(
                            "Asset (handle id: {:?}) has been loaded successfully",
                            handle,
                        );
                        load_notifier.complete();
                        raw
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

                let cooked_prefab = Self::cook_prefab(&prefab, storage, component_registry);

                prefab.prefab = Some(cooked_prefab);
                storage.update_asset(handle, prefab, version);
                if commit {
                    storage.commit_asset(handle, version);
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
                .write_resource::<AssetStorage<Prefab>>()
                .write_resource::<DefaultLoader>()
                .build(
                    move |_,
                          _,
                          (component_registry, processing_queue, prefab_storage, loader),
                          _| {
                        prefab_asset_processor(
                            component_registry,
                            processing_queue,
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
    prefab_storage: &mut AssetStorage<Prefab>,
    loader: &mut DefaultLoader,
) {
    #[cfg(feature = "profiler")]
    profile_scope!("prefab_asset_processor");

    processing_queue.process(prefab_storage, component_registry, loader);
    prefab_storage.process_custom_drop(|_| {});
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prefab::{ComponentRegistryBuilder, Prefab};
    use crate::{processor::LoadNotifier, Handle};
    use amethyst_core::ecs::World;
    use atelier_assets::loader::{
        crossbeam_channel::{unbounded, Sender},
        handle::{AssetHandle, RefOp},
        storage::{AtomicHandleAllocator, HandleAllocator},
    };
    use legion_prefab::PrefabRef;
    use std::{
        collections::HashMap,
        sync::{Arc, Once},
    };

    pub fn setup_logger() {
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{}][{}] {}",
                    record.target(),
                    record.level(),
                    message
                ))
            })
            .level(log::LevelFilter::Trace)
            .level_for("mio", log::LevelFilter::Error)
            .chain(std::io::stdout())
            .apply()
            .expect("Could not start logger");
    }

    static INIT: Once = Once::new();

    pub fn setup() {
        INIT.call_once(|| {
            setup_logger();
        });
    }

    struct Fixture {
        loader: DefaultLoader,
        processing_queue: PrefabProcessingQueue,
        prefab_storage: AssetStorage<Prefab>,
        component_registry: ComponentRegistry,
        handle_maker: HandleMaker,
    }

    impl Fixture {
        fn setup() -> Self {
            let loader = DefaultLoader::default();
            let processing_queue = PrefabProcessingQueue::default();
            let prefab_storage = AssetStorage::<Prefab>::new(loader.indirection_table.clone());
            let component_registry = ComponentRegistryBuilder::default()
                .auto_register_components()
                .build();
            let handle_allocator = Arc::new(AtomicHandleAllocator::default());
            let (ref_sender, _) = unbounded();
            let handle_maker = HandleMaker::new(handle_allocator, ref_sender);
            Self {
                loader,
                processing_queue,
                prefab_storage,
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
        setup();
        let Fixture {
            mut loader,
            mut processing_queue,
            mut prefab_storage,
            component_registry,
            handle_maker,
        } = Fixture::setup();

        let prefab_handle = handle_maker.make_handle::<Prefab>();

        let prefab_world = World::default();
        let raw_prefab = Prefab {
            raw_prefab: legion_prefab::Prefab::new(prefab_world),
            dependencies: None,
            prefab: None,
            dependers: FnvHashSet::default(),
        };
        let version = 0;

        let load_notifier = LoadNotifier::new(prefab_handle.load_handle(), None, None);
        processing_queue.enqueue_processed(
            Ok(raw_prefab),
            prefab_handle.load_handle(),
            load_notifier,
            version,
            false,
        );
        prefab_asset_processor(
            &component_registry,
            &mut processing_queue,
            &mut prefab_storage,
            &mut loader,
        );

        prefab_storage.commit_asset(prefab_handle.load_handle(), version);

        let asset = prefab_storage
            .get(&prefab_handle)
            .expect("prefab is not in storage");
        assert!(asset.prefab.is_some());
    }

    #[ignore] // FIXME: We need a MockLoader so that we can control the asset handles that are returned
    #[test]
    fn prefab_with_dependencies() {
        setup();
        let Fixture {
            mut loader,
            mut processing_queue,
            mut prefab_storage,
            component_registry,
            handle_maker,
        } = Fixture::setup();

        let prefab_handle_root = handle_maker.make_handle::<Prefab>();

        let prefab_world = World::default();
        let mut prefab_root = Prefab {
            raw_prefab: legion_prefab::Prefab::new(prefab_world),
            dependencies: None,
            prefab: None,
            dependers: FnvHashSet::default(),
        };

        let prefab_handle_1 = handle_maker.make_handle::<Prefab>();
        let prefab_world_1 = World::default();
        let prefab_1 = Prefab {
            raw_prefab: legion_prefab::Prefab::new(prefab_world_1),
            dependencies: None,
            prefab: None,
            dependers: FnvHashSet::default(),
        };
        let version = 0;
        prefab_root.raw_prefab.prefab_meta.prefab_refs.insert(
            prefab_1.raw_prefab.prefab_id(),
            PrefabRef {
                overrides: HashMap::new(),
            },
        );

        let load_notifier = LoadNotifier::new(prefab_handle_root.load_handle(), None, None);
        processing_queue.enqueue_processed(
            Ok(prefab_root),
            prefab_handle_root.load_handle(),
            load_notifier,
            version,
            false,
        );
        prefab_asset_processor(
            &component_registry,
            &mut processing_queue,
            &mut prefab_storage,
            &mut loader,
        );

        let load_notifier = LoadNotifier::new(prefab_handle_1.load_handle(), None, None);
        processing_queue.enqueue_processed(
            Ok(prefab_1),
            prefab_handle_1.load_handle(),
            load_notifier,
            version,
            false,
        );
        prefab_asset_processor(
            &component_registry,
            &mut processing_queue,
            &mut prefab_storage,
            &mut loader,
        );

        prefab_storage.commit_asset(prefab_handle_1.load_handle(), version);
        prefab_asset_processor(
            &component_registry,
            &mut processing_queue,
            &mut prefab_storage,
            &mut loader,
        );
        prefab_asset_processor(
            &component_registry,
            &mut processing_queue,
            &mut prefab_storage,
            &mut loader,
        );
        prefab_storage.commit_asset(prefab_handle_root.load_handle(), version);

        let asset = prefab_storage
            .get(&prefab_handle_root)
            .expect("prefab is not in storage");
        assert!(asset.prefab.is_some());
    }
}
