use std::{collections::HashMap, io::Read};

use atelier_assets::{
    core::AssetUuid,
    importer::{self as atelier_importer, ImportedAsset, Importer, ImporterValue},
};
use atelier_importer::ImportOp;
use legion_prefab::ComponentRegistration;
use prefab_format::ComponentTypeUuid;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

use crate::prefab::RawPrefab;

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
    ) -> atelier_importer::Result<ImporterValue> {
        log::info!("Importing prefab");
        ///////////////////////////////////////////////////////////////
        // STEP 1: Read in the data
        ///////////////////////////////////////////////////////////////

        // Read in the data
        let mut bytes = Vec::new();
        source.read_to_end(&mut bytes)?;

        ///////////////////////////////////////////////////////////////
        // STEP 2: Deserialize the prefab into a legion world
        ///////////////////////////////////////////////////////////////

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
        prefab_format::deserialize(&mut de, &prefab_deser)?;
        let raw_prefab = prefab_deser.prefab();

        let prefab_asset = RawPrefab { raw_prefab };

        ///////////////////////////////////////////////////////////////
        // STEP 3: Now we need to save it into an asset
        ///////////////////////////////////////////////////////////////

        // {
        //     let legion_world_str =
        //         ron::ser::to_string_pretty(&prefab_asset, ron::ser::PrettyConfig::default())
        //             .unwrap();

        //     log::trace!("Serialized legion world:");
        //     log::trace!("legion_world_str {}", legion_world_str);

        //     let mut ron_ser =
        //         ron::ser::Serializer::new(Some(ron::ser::PrettyConfig::default()), true);
        //     let prefab_ser = legion_prefab::PrefabFormatSerializer::new(
        //         prefab_serde_context,
        //         &prefab_asset.raw_prefab,
        //     );
        //     prefab_format::serialize(
        //         &mut ron_ser,
        //         &prefab_ser,
        //         prefab_asset.raw_prefab.prefab_id(),
        //     )
        //     .expect("failed to round-trip prefab");
        //     log::trace!(
        //         "Round-tripped legion world: {}",
        //         ron_ser.into_output_string()
        //     );
        // }

        // Add the ID to the .meta
        let prefab_id = prefab_asset.raw_prefab.prefab_id();
        state.id = Some(AssetUuid(prefab_id));

        //{
        //    //let mut ron_serializer = ron::ser::Serializer::new(Some(ron::ser::PrettyConfig::default()), true);
        //    let ron_string = ron::ser::to_string_pretty(&prefab_asset, Default::default()).unwrap();
        //    println!("{}", ron_string);
        //    let deser = ron::de::from_str::<RawPrefab>(&ron_string).unwrap();
        //    println!("ron deser complete");
        //    println!("read {} entities", deser.prefab.world.len());
        //}

        //{
        //    println!("start bincode ser");
        //    let s = bincode::serialize(&prefab_asset).unwrap();
        //    let d = bincode::deserialize::<RawPrefab>(&s).unwrap();
        //    println!("read {} entities", d.prefab.world.len());
        //}

        // let entities: Vec<Entity> = <(Entity,)>::query()
        //     .iter(&prefab_asset.raw_prefab.world)
        //     .map(|(entity,)| *entity)
        //     .collect();
        // for entity in entities.iter() {
        //     let entry = &prefab_asset.raw_prefab.world.entry(*entity).unwrap();
        //     println!("{:?}", entry.archetype())
        // }

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
