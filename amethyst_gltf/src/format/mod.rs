//! GLTF format

use std::{cmp::Ordering, collections::HashMap, sync::Arc};

use gltf::{self, Gltf};
use log::debug;
use serde::{Deserialize, Serialize};

use amethyst_animation::AnimationHierarchyPrefab;
use amethyst_assets::{Format, FormatValue, Prefab, Source};
use amethyst_core::{
    math::{convert, Quaternion, Unit, Vector3, Vector4},
    transform::Transform,
};
use amethyst_error::{format_err, Error, ResultExt};
use amethyst_rendy::{camera::CameraPrefab, light::LightPrefab};

use crate::{error, GltfMaterialSet, GltfNodeExtent, GltfPrefab, GltfSceneOptions, Named};

use self::{
    animation::load_animations,
    importer::{get_image_data, import, Buffers, ImageFormat},
    material::load_material,
    mesh::load_mesh,
    skin::load_skin,
};

mod animation;
mod importer;
mod material;
mod mesh;
mod skin;

/// Gltf scene format, will load a single scene from a Gltf file.
///
/// Using the `GltfSceneLoaderSystem` a `Handle<GltfSceneAsset>` from this format can be attached
/// to an entity in ECS, and the system will then load the full scene using the given entity
/// as the root node of the scene hierarchy.
///
/// See `GltfSceneOptions` for more information about the load options.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GltfSceneFormat(pub GltfSceneOptions);

impl Format<Prefab<GltfPrefab>> for GltfSceneFormat {
    fn name(&self) -> &'static str {
        "GLTFScene"
    }

    fn import(
        &self,
        name: String,
        source: Arc<dyn Source>,
        _create_reload: Option<Box<dyn Format<Prefab<GltfPrefab>>>>,
    ) -> Result<FormatValue<Prefab<GltfPrefab>>, Error> {
        Ok(FormatValue::data(
            load_gltf(source, &name, &self.0)
                .with_context(|_| format_err!("Failed to import gltf scene '{:?}'", name))?,
        ))
    }
}

fn load_gltf(
    source: Arc<dyn Source>,
    name: &str,
    options: &GltfSceneOptions,
) -> Result<Prefab<GltfPrefab>, Error> {
    debug!("Loading GLTF scene '{}'", name);
    import(source.clone(), name)
        .with_context(|_| error::Error::GltfImporterError)
        .and_then(|(gltf, buffers)| {
            load_data(&gltf, &buffers, options, source, name).map_err(Into::into)
        })
}

fn load_data(
    gltf: &Gltf,
    buffers: &Buffers,
    options: &GltfSceneOptions,
    source: Arc<dyn Source>,
    name: &str,
) -> Result<Prefab<GltfPrefab>, Error> {
    let scene_index = get_scene_index(gltf, options)?;
    let mut prefab = Prefab::<GltfPrefab>::new();
    load_scene(
        gltf,
        scene_index,
        buffers,
        options,
        source,
        name,
        &mut prefab,
    )?;
    Ok(prefab)
}

fn get_scene_index(gltf: &Gltf, options: &GltfSceneOptions) -> Result<usize, Error> {
    let num_scenes = gltf.scenes().len();
    match (options.scene_index, gltf.default_scene()) {
        (Some(index), _) if index >= num_scenes => {
            Err(error::Error::InvalidSceneGltf(num_scenes).into())
        }
        (Some(index), _) => Ok(index),
        (None, Some(scene)) => Ok(scene.index()),
        (None, _) if num_scenes > 1 => Err(error::Error::InvalidSceneGltf(num_scenes).into()),
        (None, _) => Ok(0),
    }
}

fn load_scene(
    gltf: &Gltf,
    scene_index: usize,
    buffers: &Buffers,
    options: &GltfSceneOptions,
    source: Arc<dyn Source>,
    name: &str,
    prefab: &mut Prefab<GltfPrefab>,
) -> Result<(), Error> {
    let scene = gltf
        .scenes()
        .nth(scene_index)
        .expect("Tried to load a scene which does not exist");
    let mut node_map = HashMap::new();
    let mut skin_map = HashMap::new();
    let mut bounding_box = GltfNodeExtent::default();
    let mut material_set = GltfMaterialSet::default();
    for node in scene.nodes() {
        let index = prefab.add(Some(0), None);
        load_node(
            gltf,
            &node,
            index,
            buffers,
            options,
            source.clone(),
            name,
            prefab,
            &mut node_map,
            &mut skin_map,
            &mut bounding_box,
            &mut material_set,
        )?;
    }
    if bounding_box.valid() {
        prefab.data_or_default(0).extent = Some(bounding_box);
    }
    prefab.data_or_default(0).materials = Some(material_set);

    // load skins
    for (node_index, skin_info) in skin_map {
        load_skin(
            &gltf.skins().nth(skin_info.skin_index).expect(
                "Unreachable: `skin_map` is initialized with indexes from the `Gltf` object",
            ),
            buffers,
            *node_map
                .get(&node_index)
                .expect("Unreachable: `node_map` should contain all nodes present in `skin_map`"),
            &node_map,
            skin_info.mesh_indices,
            prefab,
        )?;
    }

    // load animations, if applicable
    if options.load_animations {
        let mut hierarchy_prefab = AnimationHierarchyPrefab::default();
        hierarchy_prefab.nodes = node_map
            .iter()
            .map(|(node, entity)| (*node, *entity))
            .collect();
        prefab
            .data_or_default(0)
            .animatable
            .get_or_insert_with(Default::default)
            .hierarchy = Some(hierarchy_prefab);

        prefab
            .data_or_default(0)
            .animatable
            .get_or_insert_with(Default::default)
            .animation_set = Some(load_animations(gltf, buffers, &node_map)?);
    }

    Ok(())
}

#[derive(Debug)]
struct SkinInfo {
    skin_index: usize,
    mesh_indices: Vec<usize>,
}

fn load_node(
    gltf: &Gltf,
    node: &gltf::Node<'_>,
    entity_index: usize,
    buffers: &Buffers,
    options: &GltfSceneOptions,
    source: Arc<dyn Source>,
    name: &str,
    prefab: &mut Prefab<GltfPrefab>,
    node_map: &mut HashMap<usize, usize>,
    skin_map: &mut HashMap<usize, SkinInfo>,
    parent_bounding_box: &mut GltfNodeExtent,
    material_set: &mut GltfMaterialSet,
) -> Result<(), Error> {
    node_map.insert(node.index(), entity_index);

    // Load node name.
    if let Some(name) = node.name() {
        prefab.data_or_default(entity_index).name = Some(Named::new(name.to_string()));
    }

    // Load transformation data, default will be identity
    let (translation, rotation, scale) = node.transform().decomposed();
    let mut local_transform = Transform::default();
    *local_transform.translation_mut() = convert::<_, Vector3<f32>>(Vector3::from(translation));
    *local_transform.rotation_mut() = Unit::new_normalize(convert::<_, Quaternion<f32>>(
        Quaternion::from(Vector4::from(rotation)),
    ));
    *local_transform.scale_mut() = convert::<_, Vector3<f32>>(Vector3::from(scale));
    prefab.data_or_default(entity_index).transform = Some(local_transform);

    // Load camera
    if let Some(camera) = node.camera() {
        prefab.data_or_default(entity_index).camera = Some(match camera.projection() {
            gltf::camera::Projection::Orthographic(proj) => CameraPrefab::Orthographic {
                left: -proj.xmag(),
                right: proj.xmag(),
                bottom: -proj.ymag(),
                top: proj.ymag(),
                znear: proj.znear(),
                zfar: proj.zfar(),
            },
            gltf::camera::Projection::Perspective(proj) => CameraPrefab::Perspective {
                aspect: proj.aspect_ratio().ok_or_else(|| {
                    format_err!(
                        "Camera {} is a perspective projection, but has no aspect ratio",
                        camera.index()
                    )
                })?,
                fovy: proj.yfov(),
                znear: proj.znear(),
            },
        });
    }

    // Load lights
    if let Some(light) = node.light() {
        prefab.data_or_default(entity_index).light = Some(LightPrefab::from(light));
    }

    // check for skinning
    let mut skin = node.skin().map(|skin| SkinInfo {
        skin_index: skin.index(),
        mesh_indices: Vec::default(),
    });

    let mut bounding_box = GltfNodeExtent::default();

    // load graphics
    if let Some(mesh) = node.mesh() {
        let mut graphics = load_mesh(&mesh, buffers, options)?;
        match graphics.len().cmp(&1) {
            Ordering::Equal => {
                // single primitive can be loaded directly onto the node
                let (mesh, material_index, bounds) = graphics.remove(0);
                bounding_box.extend_range(&bounds);
                let prefab_data = prefab.data_or_default(entity_index);
                prefab_data.mesh = Some(mesh);
                if let Some((material_id, material)) =
                    material_index.and_then(|index| gltf.materials().nth(index).map(|m| (index, m)))
                {
                    material_set
                        .materials
                        .entry(material_id)
                        .or_insert(load_material(&material, buffers, source.clone(), name)?);
                    prefab_data.material_id = Some(material_id);
                }
                // if we have a skin we need to track the mesh entities
                if let Some(ref mut skin) = skin {
                    skin.mesh_indices.push(entity_index);
                }
            }
            Ordering::Greater => {
                // if we have multiple primitives,
                // we need to add each primitive as a child entity to the node
                for (mesh, material_index, bounds) in graphics {
                    let mesh_entity = prefab.add(Some(entity_index), None);
                    let prefab_data = prefab.data_or_default(mesh_entity);
                    prefab_data.transform = Some(Transform::default());
                    prefab_data.mesh = Some(mesh);
                    if let Some((material_id, material)) = material_index
                        .and_then(|index| gltf.materials().nth(index).map(|m| (index, m)))
                    {
                        material_set
                            .materials
                            .entry(material_id)
                            .or_insert(load_material(&material, buffers, source.clone(), name)?);
                        prefab_data.material_id = Some(material_id);
                    }

                    // if we have a skin we need to track the mesh entities
                    if let Some(ref mut skin) = skin {
                        skin.mesh_indices.push(mesh_entity);
                    }

                    // extent
                    bounding_box.extend_range(&bounds);
                    prefab_data.extent = Some(bounds.into());
                }
            }
            Ordering::Less => {}
        }
    }

    // load children
    for child in node.children() {
        let index = prefab.add(Some(entity_index), None);
        load_node(
            gltf,
            &child,
            index,
            buffers,
            options,
            source.clone(),
            name,
            prefab,
            node_map,
            skin_map,
            &mut bounding_box,
            material_set,
        )?;
    }
    if bounding_box.valid() {
        parent_bounding_box.extend(&bounding_box);
        prefab.data_or_default(entity_index).extent = Some(bounding_box);
    }

    // propagate skin information
    if let Some(skin) = skin {
        skin_map.insert(node.index(), skin);
    }

    Ok(())
}
