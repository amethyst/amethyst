use std::{cmp::Ordering, collections::HashMap, io::Read};

use amethyst_assets::{
    distill_importer,
    distill_importer::{Error, ImportOp, ImportedAsset, Importer, ImporterValue},
    make_handle,
    prefab::{
        legion_prefab,
        legion_prefab::{CookedPrefab, PrefabBuilder},
        Prefab,
    },
    AssetUuid, Handle,
};
use amethyst_core::{
    ecs::{Entity, World},
    math::{convert, Quaternion, Unit, Vector3, Vector4},
    transform::Transform,
};
use amethyst_rendy::{light::Light, types::MeshData, Camera};
use gltf::{buffer, buffer::Data, Document, Gltf, Mesh, Node};
use log::debug;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

use crate::{
    importer::{
        gltf_bytes_converter::convert_bytes, images::load_image, material::load_material,
        mesh::load_mesh,
    },
    types::MeshHandle,
    GltfNodeExtent, GltfSceneOptions,
};

mod gltf_bytes_converter;
mod images;
mod material;
mod mesh;

#[derive(Debug)]
struct SkinInfo {
    skin_index: usize,
    mesh_indices: Vec<usize>,
}

/// A simple state for Importer to retain the same UUID between imports
/// for all single-asset source files
#[derive(Default, Deserialize, Serialize, TypeUuid)]
#[uuid = "3c5571c0-abec-436e-9b28-6bce92f1070a"]
pub struct GltfImporterState {
    pub id: Option<AssetUuid>,
    pub images_uuids: Option<HashMap<usize, AssetUuid>>,
    pub material_uuids: Option<HashMap<String, AssetUuid>>,
    pub mesh_uuids: Option<HashMap<String, AssetUuid>>,
}

/// The importer for '.gltf' or '.glb' files.
#[derive(Default, TypeUuid)]
#[uuid = "6dbb4496-bd73-42cd-b817-11046e964e30"]
pub struct GltfImporter;

impl Importer for GltfImporter {
    fn version_static() -> u32 {
        1
    }

    fn version(&self) -> u32 {
        Self::version_static()
    }

    type Options = GltfSceneOptions;
    type State = GltfImporterState;

    fn import(
        &self,
        op: &mut ImportOp,
        source: &mut dyn Read,
        options: &Self::Options,
        state: &mut Self::State,
    ) -> amethyst_assets::distill_importer::Result<ImporterValue> {
        log::info!("Importing scene");
        let mut asset_accumulator: Vec<ImportedAsset> = Vec::new();
        let mut world = World::default();

        let mut bytes = Vec::new();
        source.read_to_end(&mut bytes)?;
        let result = convert_bytes(&bytes);

        if let Err(err) = result {
            log::error!("GLTF Import error: {:?}", err);
            return Err(distill_importer::Error::Boxed(Box::new(err)));
        }

        let (doc, buffers, _images) = result.unwrap();

        doc.images().for_each(|image| {
            let mut image_assets = load_image(&image, state, &buffers);
            asset_accumulator.append(&mut image_assets);
        });

        doc.materials().for_each(|material| {
            let mut material_assets = load_material(&material, op, &buffers, state);
            asset_accumulator.append(&mut material_assets);
        });

        // TODO : load animation

        let scene_index = get_scene_index(&doc, options).expect("No scene has been found !");
        let scene = doc
            .scenes()
            .nth(scene_index)
            .expect("Tried to load a scene which does not exist");

        scene.nodes().into_iter().for_each(|node| {
            let mut node_assets = load_node(&node, &mut world, op, state, &options, &buffers, None);
            asset_accumulator.append(&mut node_assets);
        });

        let legion_prefab = legion_prefab::Prefab::new(world);
        let scene_prefab = Prefab::new(legion_prefab);

        if state.id.is_none() {
            state.id = Some(op.new_asset_uuid());
        }

        asset_accumulator.push(ImportedAsset {
            id: state
                .id
                .expect("UUID generation for main scene prefab didn't work"),
            search_tags: Vec::new(),
            build_deps: Vec::new(),
            load_deps: Vec::new(),
            asset_data: Box::new(scene_prefab),
            build_pipeline: None,
        });

        Ok(ImporterValue {
            assets: asset_accumulator,
        })
    }
}

// This method will return the mesh assets that need to be loaded by distill.
// Contrary to the material, I've considered that loading a mesh is only mandatory if it's used somewhere
// For example, if I need to attach a mesh, I'll add the mesh as an ImportedAsset and set
// the matching uuid to the entry's component in an Handle
fn load_node(
    node: &Node<'_>,
    world: &mut World,
    op: &mut ImportOp,
    state: &mut GltfImporterState,
    options: &GltfSceneOptions,
    buffers: &Vec<Data>,
    parent_node_entity: Option<&Entity>,
) -> Vec<ImportedAsset> {
    let current_node_entity = world.push(());
    let mut imported_assets = Vec::new();

    if let Some(transform) = load_transform(node) {
        if let Some(p) = parent_node_entity {
            let t = {
                let entry = world.entry(*p).expect("We just added this entity");
                let result = entry.get_component::<Transform>();
                if let Ok(result) = result {
                    *result
                } else {
                    transform
                }
            };
            world
                .entry(current_node_entity)
                .expect("We just added this entity")
                .add_component(t);
        } else {
            world
                .entry(current_node_entity)
                .expect("We just added this entity")
                .add_component(transform);
        }
    }

    if let Some(camera) = load_camera(node) {
        debug!("Adding a camera component to to the current node entity and has parent ?");
        world
            .entry(current_node_entity)
            .expect("We just added this entity")
            .add_component(camera);
    }

    if let Some(light) = load_light(node) {
        debug!("Adding a light component to to the current node entity");
        world
            .entry(current_node_entity)
            .expect("We just added this entity")
            .add_component(light);
    }

    let mut skin = node.skin().map(|skin| {
        SkinInfo {
            skin_index: skin.index(),
            mesh_indices: Vec::default(),
        }
    });

    let mut bounding_box = GltfNodeExtent::default();

    // load graphics
    if let Some(mesh) = node.mesh() {
        let mut loaded_mesh = load_mesh(&mesh, buffers, options).expect("It should work");
        match loaded_mesh.len().cmp(&1) {
            Ordering::Equal => {
                // single primitive can be loaded directly onto the node
                let (name, mesh, _material_index, bounds) = loaded_mesh.remove(0);
                bounding_box.extend_range(&bounds);

                if state.mesh_uuids.is_none() {
                    state.mesh_uuids = Some(Default::default());
                }

                let mesh_asset_id = *state
                    .mesh_uuids
                    .as_mut()
                    .expect("Meshes hashmap didn't work")
                    .entry(name)
                    .or_insert_with(|| op.new_asset_uuid());

                let mesh_data: MeshData = mesh.into();
                imported_assets.push(ImportedAsset {
                    id: mesh_asset_id,
                    search_tags: vec![],
                    build_deps: vec![],
                    load_deps: vec![],
                    build_pipeline: None,
                    asset_data: Box::new(mesh_data),
                });

                world
                    .entry(current_node_entity)
                    .expect("We just added this entity")
                    .add_component(MeshHandle(make_handle(mesh_asset_id)));

                debug!("Adding a mesh component to to the current node entity");
                //TODO: material and skin

                // if we have a skin we need to track the mesh entities
                if let Some(_skin) = skin {
                    //skin.mesh_indices.push(mesh_asset.index);
                    // TODO
                }
            }
            Ordering::Greater => {
                // if we have multiple primitives,
                // we need to add each primitive as a child entity to the node
                // TODO reimplement here
            }
            Ordering::Less => {
                // Nothing to do here
            }
        }
    }

    // load childs
    for child in node.children() {
        let mut child_assets = load_node(
            &child,
            world,
            op,
            state,
            options,
            buffers,
            Some(&current_node_entity),
        );
        imported_assets.append(&mut child_assets);
    }

    imported_assets
}

fn load_light(node: &Node<'_>) -> Option<Light> {
    if let Some(light) = node.light() {
        return Some(Light::from(light));
    }
    None
}

fn load_camera(node: &Node<'_>) -> Option<Camera> {
    if let Some(camera) = node.camera() {
        return Some(match camera.projection() {
            gltf::camera::Projection::Orthographic(proj) => {
                Camera::orthographic(
                    -proj.xmag(),
                    proj.xmag(),
                    -proj.ymag(),
                    proj.ymag(),
                    proj.znear(),
                    proj.zfar(),
                )
            }
            gltf::camera::Projection::Perspective(proj) => {
                Camera::perspective(
                    proj.aspect_ratio().expect("Camera {} failed to load"),
                    proj.yfov(),
                    proj.znear(),
                )
            }
        });
    }
    None
}

fn load_transform(node: &Node<'_>) -> Option<Transform> {
    // Load transformation data, default will be identity
    let (translation, rotation, scale) = node.transform().decomposed();
    let mut local_transform = Transform::default();

    *local_transform.translation_mut() = convert::<_, Vector3<f32>>(Vector3::from(translation));
    *local_transform.rotation_mut() = Unit::new_normalize(convert::<_, Quaternion<f32>>(
        Quaternion::from(Vector4::from(rotation)),
    ));
    *local_transform.scale_mut() = convert::<_, Vector3<f32>>(Vector3::from(scale));
    Some(local_transform)
}

fn get_scene_index(document: &Document, options: &GltfSceneOptions) -> Result<usize, Error> {
    let num_scenes = document.scenes().len();
    match (options.scene_index, document.default_scene()) {
        (Some(index), _) if index >= num_scenes => {
            Err(Error::Custom(format!("Invalid Scene Gltf {}", num_scenes)))
        }
        (Some(index), _) => Ok(index),
        (None, Some(scene)) => Ok(scene.index()),
        (None, _) if num_scenes > 1 => {
            Err(Error::Custom(format!("Invalid Scene Gltf {}", num_scenes)))
        }
        (None, _) => Ok(0),
    }
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Read};

    use atelier_assets::importer::BoxedImporter;
    use type_uuid::TypeUuid;

    use super::{super::GltfSceneOptions, *};

    #[test]
    fn importer_basic_test() {
        let mut f = File::open("test/sample.gltf").expect("suzanne.glb not found");
        let mut buffer = Vec::new();
        // read the whole file
        f.read_to_end(&mut buffer)
            .expect("read_to_end did not work");
        let mut buffer_slice = buffer.as_slice();
        let importer: Box<dyn BoxedImporter> = Box::new(GltfImporter::default());
        let mut import_op = ImportOp::default();
        let res = futures::executor::block_on(importer.import_boxed(
            &mut import_op,
            &mut buffer_slice,
            Box::new(GltfSceneOptions::default()),
            Box::new(GltfImporterState { id: None }),
        ));
        match res {
            Ok(r) => {
                println!("res : {:?}", r.value.assets.len());
            }
            Err(e) => {
                println!("error e {:?}", e);
            }
        };
    }
}
