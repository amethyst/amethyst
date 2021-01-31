use std::{collections::HashMap, io::Read};

use distill::{
    core::AssetUuid,
    importer::{self as distill_importer, ImportOp, ImportedAsset, Importer, ImporterValue},
};
use legion_prefab::ComponentRegistration;
use prefab_format::ComponentTypeUuid;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

use crate::prefab::Prefab;

#[derive(Default, Deserialize, Serialize, TypeUuid, Clone, Copy)]
#[uuid = "80583980-24d4-4034-8394-ea749b43f55d"]
pub struct PrefabImporterOptions {}

/// A simple state for Importer to retain the same UUID between imports
/// for all single-asset source files
#[derive(Default, Deserialize, Serialize, TypeUuid)]
#[uuid = "14d89614-7e10-4f59-952f-af32c73bda90"]
pub struct PrefabImporterState {
    pub id: Option<AssetUuid>,
}

/// The importer for '.prefab' files.
#[derive(Default, TypeUuid)]
#[uuid = "5bdf4d06-a1cb-437b-b182-d6d8cb23512c"]
pub struct PrefabImporter {}

impl Importer for PrefabImporter {
    type State = PrefabImporterState;
    type Options = PrefabImporterOptions;

    fn version_static() -> u32 {
        1
    }

    fn version(&self) -> u32 {
        Self::version_static()
    }

    fn import(
        &self,
        _op: &mut ImportOp,
        source: &mut dyn Read,
        _: &Self::Options,
        state: &mut Self::State,
    ) -> distill_importer::Result<ImporterValue> {
        log::info!("Importing prefab");
        // STEP 1: Read in the data

        // Read in the data
        let mut bytes = Vec::new();
        source.read_to_end(&mut bytes)?;

        // STEP 2: Deserialize the prefab into a legion world

        // Create a deserializer
        let mut de = ron::de::Deserializer::from_bytes(bytes.as_slice()).unwrap();

        // Create the component registry
        let registered_components = {
            let comp_registrations = legion_prefab::iter_component_registrations();
            log::info!("Getting registered components");
            let component_types: HashMap<ComponentTypeUuid, ComponentRegistration> =
                comp_registrations
                    .map(|reg| (*reg.uuid(), reg.clone()))
                    .collect();

            component_types
        };

        let prefab_serde_context = legion_prefab::PrefabSerdeContext {
            registered_components: &registered_components,
        };

        let prefab_deser = legion_prefab::PrefabFormatDeserializer::new(prefab_serde_context);
        if let Err(err) = prefab_format::deserialize(&mut de, &prefab_deser) {
            return Err(distill_importer::Error::RonDe(err));
        }
        let raw_prefab = prefab_deser.prefab();

        let prefab_asset = Prefab {
            raw: raw_prefab,
            ..Default::default()
        };

        // STEP 3: Now we need to save it into an asset

        // Add the ID to the .meta
        let prefab_id = prefab_asset.raw.prefab_id();
        state.id = Some(AssetUuid(prefab_id));

        Ok(ImporterValue {
            assets: vec![ImportedAsset {
                id: state.id.expect("AssetUuid not generated"),
                search_tags: Vec::new(),
                build_deps: Vec::new(),
                load_deps: Vec::new(),
                asset_data: Box::new(prefab_asset),
                build_pipeline: None,
            }],
        })
    }
}
