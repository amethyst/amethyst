use gltf::{Gltf, Document, buffer, Node, Mesh};
use gltf::buffer::Data;
use gltf::iter::Buffers;
use atelier_assets::importer::{Importer, ImportOp, ImporterValue, Error, ImportedAsset};
use std::io::Read;
use crate::{GltfSceneOptions, error, GltfAsset, GltfNodeExtent};
use atelier_assets::core::AssetUuid;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;
use amethyst_assets::atelier_importer;
use atelier_assets::make_handle;
use atelier_assets::loader::handle::Handle;
use crate::importer::gltf_bytes_converter::convert_bytes;
use amethyst_core::transform::Transform;
use amethyst_core::math::{convert, Vector3, Unit, Quaternion, Vector4};
use amethyst_core::Named;
use amethyst_rendy::Camera;
use amethyst_rendy::light::Light;
use gltf::khr_lights_punctual::Kind;
use amethyst_animation::Skin;
use crate::importer::mesh::load_mesh;
use std::cmp::Ordering;

mod gltf_bytes_converter;
mod mesh;

#[derive(Debug)]
struct SkinInfo {
    skin_index: usize,
    mesh_indices: Vec<usize>,
}


#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GltfObjectId {
    Name(String),
    Index(usize),
}

/// A simple state for Importer to retain the same UUID between imports
/// for all single-asset source files
#[derive(Default, Deserialize, Serialize, TypeUuid)]
#[uuid = "3c5571c0-abec-436e-9b28-6bce92f1070a"]
pub struct GltfImporterState {
    pub id: Option<AssetUuid>,
}

/// The importer for '.gltf' or '.glb' files.
#[derive(Default, TypeUuid)]
#[uuid = "6dbb4496-bd73-42cd-b817-11046e964e30"]
pub struct GltfImporter {}

impl Importer for GltfImporter {
    fn version_static() -> u32 {
        1
    }

    fn version(&self) -> u32 {
        Self::version_static()
    }

    type Options = GltfSceneOptions;
    type State = GltfImporterState;

    fn import(&self, op: &mut ImportOp, source: &mut dyn Read, options: &Self::Options, state: &mut Self::State) -> atelier_importer::Result<ImporterValue> {
        log::info!("Importing scene");

        let mut bytes = Vec::new();
        source.read_to_end(&mut bytes)?;
        let result = convert_bytes(&bytes);

        if let Err(err) = result {
            log::error!("GLTF Import error: {:?}", err);
            return Err(Error::Boxed(Box::new(err)));
        }

        let mut asset_accumulator = Vec::new();

        let (doc, buffers, images) = result.unwrap();

        let scene_index = get_scene_index(&doc, options).expect("No scene has been found !");
        let scene = doc
            .scenes()
            .nth(scene_index)
            .expect("Tried to load a scene which does not exist");

        scene.nodes().into_iter().for_each(|node| {
            if let Some(asset) = load_node(&node, op, &options, &buffers) {
                asset_accumulator.push(asset);
            }
        });

        Ok(ImporterValue {
            assets: asset_accumulator
        })
    }
}

fn load_node(node: &Node, op: &mut ImportOp, options: &GltfSceneOptions, buffers: &Vec<Data>) -> Option<ImportedAsset> {
    let mut node_asset = GltfAsset::default();
    let mut search_tags: Vec<(String, Option<String>)> = vec![];

    if let Some(name) = node.name() {
        node_asset.name = Some(Named::new(name.to_string()));
        search_tags.push(("node_name".to_string(), Some(name.to_string())));
    }
    node_asset.transform = Some(load_transform(node));
    node_asset.camera = load_camera(node);
    node_asset.light = load_light(node);

    let mut skin = node.skin().map(|skin| SkinInfo {
        skin_index: skin.index(),
        mesh_indices: Vec::default(),
    });

    let mut bounding_box = GltfNodeExtent::default();

    // load graphics
    if let Some(mesh) = node.mesh() {
        let mut loaded_mesh = load_mesh(&mesh, buffers, options).expect("It should work");
        match loaded_mesh.len().cmp(&1) {
            Ordering::Equal => {
                // single primitive can be loaded directly onto the node
            }
            Ordering::Greater => {
                // if we have multiple primitives,
                // we need to add each primitive as a child entity to the node
            }
            Ordering::Less => {
                // Nothing to do here
            }
        }
    }

    // load childs
    for child in node.children() {
       let n = load_node(&child, op, options, buffers);
    }


    Some(ImportedAsset {
        id: op.new_asset_uuid(),
        search_tags,
        build_deps: vec![],
        load_deps: vec![],
        build_pipeline: None,
        asset_data: Box::new(node_asset),
    })
}

// TODO:  Experimental, can't test with blender export for now
fn load_light(node: &Node) -> Option<Light> {
    if let Some(light) = node.light() {
        return Some(Light::from(light));
    }
    None
}

fn load_camera(node: &Node) -> Option<Camera> {
    if let Some(camera) = node.camera() {
        return Some(
            match camera.projection() {
                gltf::camera::Projection::Orthographic(proj) => Camera::orthographic(
                    -proj.xmag(),
                    proj.xmag(),
                    -proj.ymag(),
                    proj.ymag(),
                    proj.znear(),
                    proj.zfar(),
                ),
                gltf::camera::Projection::Perspective(proj) => Camera::perspective(
                    proj.aspect_ratio().expect("Camera {} failed to load"),
                    proj.yfov(),
                    proj.znear(),
                ),
            }
        );
    }
    None
}

fn load_transform(node: &Node) -> Transform {
    // Load transformation data, default will be identity
    let (translation, rotation, scale) = node.transform().decomposed();
    let mut local_transform = Transform::default();

    *local_transform.translation_mut() = convert::<_, Vector3<f32>>(Vector3::from(translation));
    *local_transform.rotation_mut() = Unit::new_normalize(convert::<_, Quaternion<f32>>(
        Quaternion::from(Vector4::from(rotation)),
    ));
    *local_transform.scale_mut() = convert::<_, Vector3<f32>>(Vector3::from(scale));
    local_transform
}

fn get_scene_index(document: &Document, options: &GltfSceneOptions) -> Result<usize, Error> {
    let num_scenes = document.scenes().len();
    match (options.scene_index, document.default_scene()) {
        (Some(index), _) if index >= num_scenes => {
            Err(Error::Custom(format!("Invalid Scene Gltf {}", num_scenes)))
        }
        (Some(index), _) => Ok(index),
        (None, Some(scene)) => Ok(scene.index()),
        (None, _) if num_scenes > 1 => Err(Error::Custom(format!("Invalid Scene Gltf {}", num_scenes))),
        (None, _) => Ok(0),
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use super::super::GltfSceneOptions;
    use std::fs::File;
    use std::io::Read;
    use atelier_assets::importer::BoxedImporter;
    use type_uuid::TypeUuid;

    #[test]
    fn importer_basic_test() {
        let mut f = File::open("test/sample.gltf").expect("suzanne.glb not found");
        let mut buffer = Vec::new();
        // read the whole file
        f.read_to_end(&mut buffer).expect("read_to_end did not work");
        let mut buffer_slice = buffer.as_slice();
        let importer: Box<dyn BoxedImporter> = Box::new(GltfImporter::default());
        let mut import_op = ImportOp::default();
        let res = futures::executor::block_on(importer.import_boxed(&mut import_op, &mut buffer_slice, Box::new(GltfSceneOptions::default()), Box::new(GltfImporterState { id: None })));
        match res {
            Ok(r) => { println!("res : {:?}", r.value.assets.len()); }
            Err(e) => { println!("error e {:?}", e); }
        };
    }
}