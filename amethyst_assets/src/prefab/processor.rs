use amethyst_core::{
    dispatcher::System,
    ecs::{systems::ParallelRunnable, SystemBuilder},
};
use distill::core::AssetUuid;
use fnv::{FnvHashMap, FnvHashSet};
use prefab_format::PrefabUuid;

use crate::{
    self as amethyst_assets,
    loader::{DefaultLoader, Loader},
    prefab::{ComponentRegistry, Prefab},
    storage::AssetStorage,
    AssetHandle, ProcessingQueue, ProcessingState, WeakHandle,
};

crate::register_asset_type!(Prefab => Prefab; PrefabProcessorSystem);

impl Prefab {
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

        let mut dependency_stack = vec![(prefab, prefab.dependencies.iter())];

        while let Some((cur_prefab, children)) = dependency_stack.last_mut() {
            if let Some(child_handle) = children.next() {
                log::debug!("Checking for child prefab {:?}", child_handle);
                if let Some(child_prefab) = storage.get(child_handle) {
                    if prefab_lookup.contains_key(&child_prefab.raw.prefab_id()) {
                        continue;
                    }

                    dependency_stack.push((child_prefab, child_prefab.dependencies.iter()));
                } else {
                    log::error!("Prefab dependency is not yet loaded!");
                }
            } else {
                // No more dependencies, add cur_prefab to prefab_cook_order and
                // pop the stack.
                prefab_cook_order.push(cur_prefab.raw.prefab_id());
                prefab_lookup.insert(cur_prefab.raw.prefab_id(), &cur_prefab.raw);
                dependency_stack.pop();
            }
        }

        log::debug!("prefab_cook_order: {:x?}", prefab_cook_order);
        log::debug!("prefab_lookup: {:x?}", prefab_lookup.keys());

        legion_prefab::cook_prefab(
            component_registry.components(),
            component_registry.components_by_uuid(),
            prefab_cook_order.as_slice(),
            &prefab_lookup,
        )
    }
}

#[derive(Default)]
struct PrefabProcessorSystem;

impl System for PrefabProcessorSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("PrefabProcessorSystem")
                .read_resource::<ComponentRegistry>()
                .write_resource::<ProcessingQueue<Prefab>>()
                .write_resource::<AssetStorage<Prefab>>()
                .write_resource::<DefaultLoader>()
                .build(
                    move |_, _, (component_registry, processing_queue, storage, loader), _| {
                        prefab_asset_processor(
                            component_registry,
                            processing_queue,
                            storage,
                            loader,
                        );
                    },
                ),
        )
    }
}

fn prefab_asset_processor(
    component_registry: &ComponentRegistry,
    processing_queue: &mut ProcessingQueue<Prefab>,
    storage: &mut AssetStorage<Prefab>,
    loader: &mut DefaultLoader,
) -> Vec<crate::Handle<Prefab>> {
    // Re-cook prefabs with changed dependencies.
    // FIXME: deal with cyclic and diamond dependencies correctly
    let mut visited = FnvHashSet::default();

    while let Some(dependee) = processing_queue.changed.pop() {
        log::debug!("Prefab Changed: {:?}", dependee);
        let updates: Vec<(WeakHandle, legion_prefab::CookedPrefab)> = storage
            .get_for_load_handle(dependee)
            .iter()
            .flat_map(|p| p.dependers.iter())
            .flat_map(|weak_handle| {
                storage
                    .get_asset_with_version(weak_handle)
                    .map(move |(prefab, _)| (weak_handle, prefab))
            })
            .map(|(weak_handle, prefab)| {
                if visited.insert(weak_handle.load_handle()) {
                    processing_queue.changed.push(weak_handle.load_handle());
                }
                (
                    weak_handle.clone(),
                    Prefab::cook_prefab(prefab, storage, component_registry),
                )
            })
            .collect();

        use crate::storage::MutateAssetInStorage;
        for (handle, cooked_prefab) in updates.into_iter() {
            storage.mutate_asset_in_storage(&handle, move |prefab| {
                prefab.cooked = Some(cooked_prefab);
                prefab.version += 1;
            });
        }
    }

    let mut loading = Vec::new();

    processing_queue.process(storage, |mut prefab, storage, handle| {
        log::debug!("Processing Prefab {:x?}", AssetUuid(prefab.raw.prefab_id()));

        prefab.dependencies = prefab
            .raw
            .prefab_meta
            .prefab_refs
            .iter()
            .map(|(child_prefab_id, _)| {
                let handle = loader.load_asset(AssetUuid(*child_prefab_id));
                loading.push(handle.clone());
                handle
            })
            .collect();

        Ok(
            if prefab
                .dependencies
                .iter()
                .all(|handle| storage.contains(handle.load_handle()))
            {
                prefab.cooked = Some(Prefab::cook_prefab(&prefab, storage, component_registry));
                prefab.version += storage
                    .get_for_load_handle(*handle)
                    .map_or(1, |Prefab { version, .. }| *version + 1);

                ProcessingState::Loaded(prefab)
            } else {
                ProcessingState::Loading(prefab)
            },
        )
    });
    storage.process_custom_drop(|_| {});
    loading
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Once};

    use amethyst_core::{Logger, LoggerConfig};
    use distill::loader::handle::AssetHandle;
    use legion_prefab::PrefabRef;
    use serial_test::serial;

    use super::*;
    use crate::{
        prefab::{ComponentRegistryBuilder, Prefab},
        processor::LoadNotifier,
        Handle,
    };

    struct Fixture {
        loader: DefaultLoader,
        processing_queue: ProcessingQueue<Prefab>,
        prefab_storage: AssetStorage<Prefab>,
        component_registry: ComponentRegistry,
    }

    static INIT: Once = Once::new();

    impl Fixture {
        fn setup() -> Self {
            INIT.call_once(|| {
                Logger::from_config(LoggerConfig {
                    level_filter: log::LevelFilter::Trace,
                    ..Default::default()
                })
                .start();
            });

            let loader = DefaultLoader::default();
            let processing_queue = ProcessingQueue::default();
            let prefab_storage = AssetStorage::<Prefab>::new(loader.indirection_table.clone());
            let component_registry = ComponentRegistryBuilder::default()
                .auto_register_components()
                .build();

            Self {
                loader,
                processing_queue,
                prefab_storage,
                component_registry,
            }
        }
    }

    #[serial]
    #[test]
    fn prefab_is_cooked() {
        let Fixture {
            mut loader,
            mut processing_queue,
            mut prefab_storage,
            component_registry,
        } = Fixture::setup();

        let raw_prefab = Prefab::default();

        let prefab_handle: Handle<Prefab> =
            loader.load_from_data(raw_prefab, (), &processing_queue);

        prefab_asset_processor(
            &component_registry,
            &mut processing_queue,
            &mut prefab_storage,
            &mut loader,
        );

        let asset = prefab_storage
            .get(&prefab_handle)
            .expect("prefab is not in storage");
        assert!(asset.cooked.is_some());
    }

    #[serial]
    #[test]
    fn prefab_with_dependencies() {
        let Fixture {
            mut loader,
            mut processing_queue,
            mut prefab_storage,
            component_registry,
        } = Fixture::setup();

        let mut prefab_root = Prefab::default();
        let prefab_child = Prefab::default();

        // add prefab_child to dependencies of prefab_root
        prefab_root.raw.prefab_meta.prefab_refs.insert(
            prefab_child.raw.prefab_id(),
            PrefabRef {
                overrides: HashMap::new(),
            },
        );

        let prefab_handle: Handle<Prefab> =
            loader.load_from_data(prefab_root, (), &processing_queue);

        let children_handles = prefab_asset_processor(
            &component_registry,
            &mut processing_queue,
            &mut prefab_storage,
            &mut loader,
        );

        let child_handle = children_handles.get(0).unwrap().load_handle();

        processing_queue.enqueue_processed(
            Ok(prefab_child),
            child_handle,
            LoadNotifier::new(child_handle, None, None),
            0,
            false,
        );

        prefab_asset_processor(
            &component_registry,
            &mut processing_queue,
            &mut prefab_storage,
            &mut loader,
        );

        prefab_storage.commit_asset(child_handle, 0);

        prefab_asset_processor(
            &component_registry,
            &mut processing_queue,
            &mut prefab_storage,
            &mut loader,
        );

        let asset = prefab_storage
            .get(&prefab_handle)
            .expect("prefab is not in storage");
        assert!(asset.cooked.is_some());
    }
}
