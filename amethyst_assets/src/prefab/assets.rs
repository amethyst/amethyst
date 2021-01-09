use std::collections::HashMap;

use amethyst_core::ecs::*;
use atelier_assets::importer as atelier_importer;
use atelier_importer::{typetag, SerdeImportable};
use serde::{Deserialize, Serialize};
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;
use type_uuid::TypeUuid;

use crate::{
    asset::Asset, prefab::ComponentRegistry, register_asset_type, AssetStorage, ProcessingQueue,
    ProcessingState,
};

/// Cooked Prefab, containing a World with Entities
#[derive(TypeUuid, Serialize, Deserialize, SerdeImportable)]
#[uuid = "5e751ea4-e63b-4192-a008-f5bf8674e45b"]
pub struct Prefab {
    /// contains Legion World and Entity Mappings
    pub prefab: legion_prefab::CookedPrefab,
}

/// Raw prefab type, used to generate the cooked prefab
#[derive(TypeUuid, Serialize, Deserialize, SerdeImportable)]
#[uuid = "c77ccda8-f2f0-4a7f-91ef-f38fabc0e6ce"]
pub struct RawPrefab {
    /// Contains World to cook and references to other prefabs
    pub raw_prefab: legion_prefab::Prefab,
}

impl Asset for Prefab {
    fn name() -> &'static str {
        "PREFAB"
    }
    type Data = RawPrefab;
}

register_asset_type!(crate; RawPrefab => Prefab; PrefabAssetProcessor);

#[derive(Default)]
struct PrefabAssetProcessor;

impl System<'static> for PrefabAssetProcessor {
    fn build(&'static mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("PrefabAssetProcessorSystem")
                .read_resource::<ComponentRegistry>()
                .write_resource::<ProcessingQueue<RawPrefab>>()
                .write_resource::<AssetStorage<Prefab>>()
                .build(
                    move |_, _, (component_registry, processing_queue, prefab_storage), _| {
                        #[cfg(feature = "profiler")]
                        profile_scope!("prefab_asset_processor");

                        processing_queue.process(prefab_storage, |RawPrefab { raw_prefab }| {
                            let prefab_cook_order = vec![raw_prefab.prefab_id()];
                            let mut prefab_lookup = HashMap::new();
                            prefab_lookup.insert(raw_prefab.prefab_id(), &raw_prefab);

                            let prefab = legion_prefab::cook_prefab(
                                component_registry.components(),
                                component_registry.components_by_uuid(),
                                prefab_cook_order.as_slice(),
                                &prefab_lookup,
                            );

                            Ok(ProcessingState::Loaded(Prefab { prefab }))
                        });
                        prefab_storage.process_custom_drop(|_| {});
                    },
                ),
        )
    }
}
