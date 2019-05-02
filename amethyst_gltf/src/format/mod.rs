//! GLTF format

use std::{collections::HashMap, fmt::Debug, sync::Arc};

use gltf::{self, Gltf};
use log::debug;
use num_traits::NumCast;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use amethyst_animation::AnimationHierarchyPrefab;
use amethyst_assets::{Format, FormatValue, Prefab, Source};
use amethyst_core::{
    math::{Quaternion, RealField, Unit, Vector3},
    transform::Transform,
};
use amethyst_error::{format_err, Error, ResultExt};

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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GltfSceneFormat;

impl<
        N: Clone + Debug + Default + DeserializeOwned + Serialize + NumCast + RealField + From<f32>,
    > Format<Prefab<GltfPrefab<N>>> for GltfSceneFormat
{
    const NAME: &'static str = "GLTFScene";

    type Options = GltfSceneOptions;

    fn import(
        &self,
        name: String,
        source: Arc<dyn Source>,
        options: GltfSceneOptions,
        _create_reload: bool,
    ) -> Result<FormatValue<Prefab<GltfPrefab<N>>>, Error> {
        Ok(FormatValue::data(
            load_gltf(source, &name, options)
                .with_context(|_| format_err!("Failed to import gltf scene"))?,
        ))
    }
}

fn load_gltf<
    N: Clone + Debug + Default + DeserializeOwned + Serialize + NumCast + RealField + From<f32>,
>(
    source: Arc<dyn Source>,
    name: &str,
    options: GltfSceneOptions,
) -> Result<Prefab<GltfPrefab<N>>, Error> {
    debug!("Loading GLTF scene {}", name);
    import(source.clone(), name)
        .with_context(|_| error::Error::GltfImporterError)
        .and_then(|(gltf, buffers)| {
            load_data(&gltf, &buffers, &options, source, name).map_err(Into::into)
        })
}

fn load_data<
    N: Clone + Debug + Default + DeserializeOwned + Serialize + NumCast + RealField + From<f32>,
>(
    gltf: &Gltf,
    buffers: &Buffers,
    options: &GltfSceneOptions,
    source: Arc<dyn Source>,
    name: &str,
) -> Result<Prefab<GltfPrefab<N>>, Error> {
    let scene_index = get_scene_index(gltf, options)?;
    let mut prefab = Prefab::<GltfPrefab<N>>::new();
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

fn load_scene<
    N: Clone + Debug + Default + DeserializeOwned + Serialize + NumCast + RealField + From<f32>,
>(
    gltf: &Gltf,
    scene_index: usize,
    buffers: &Buffers,
    options: &GltfSceneOptions,
    source: Arc<dyn Source>,
    name: &str,
    prefab: &mut Prefab<GltfPrefab<N>>,
) -> Result<(), Error> {
    let scene = gltf
        .scenes()
        .nth(scene_index)
        .expect("Tried to load a scene which does not exist");
    let mut node_map = HashMap::new();
    let mut skin_map = HashMap::new();
    let mut bounding_box = GltfNodeExtent::default();
    let mut material_set = GltfMaterialSet::default();
    if scene.nodes().len() == 1 {
        load_node(
            gltf,
            &scene
                .nodes()
                .next()
                .expect("Unreachable: Length of nodes in scene is checked to be equal to one"),
            0,
            buffers,
            options,
            source,
            name,
            prefab,
            &mut node_map,
            &mut skin_map,
            &mut bounding_box,
            &mut material_set,
        )?;
    } else {
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
            prefab.data_or_default(0).extent = Some(bounding_box.clone());
        }
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

fn load_node<
    N: Clone + Debug + Default + DeserializeOwned + Serialize + NumCast + RealField + From<f32>,
>(
    gltf: &Gltf,
    node: &gltf::Node<'_>,
    entity_index: usize,
    buffers: &Buffers,
    options: &GltfSceneOptions,
    source: Arc<dyn Source>,
    name: &str,
    prefab: &mut Prefab<GltfPrefab<N>>,
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
    let mut local_transform = Transform::<N>::default();
    *local_transform.translation_mut() = Vector3::new(
        translation[0].into(),
        translation[1].into(),
        translation[2].into(),
    );
    // gltf quat format: [x, y, z, w], argument order expected by our quaternion: (w, x, y, z)
    *local_transform.rotation_mut() = Unit::new_normalize(Quaternion::new(
        rotation[3].into(),
        rotation[0].into(),
        rotation[1].into(),
        rotation[2].into(),
    ));
    *local_transform.scale_mut() = Vector3::new(scale[0].into(), scale[1].into(), scale[2].into());
    prefab.data_or_default(entity_index).transform = Some(local_transform);

    // check for skinning
    let mut skin = node.skin().map(|skin| SkinInfo {
        skin_index: skin.index(),
        mesh_indices: Vec::default(),
    });

    let mut bounding_box = GltfNodeExtent::default();

    // load graphics
    if let Some(mesh) = node.mesh() {
        let mut graphics = load_mesh(&mesh, buffers, options)?;
        if graphics.len() == 1 {
            // single primitive can be loaded directly onto the node
            let (mesh, material_index, bounds) = graphics.remove(0);
            bounding_box.extend_range(&bounds);
            let prefab_data = prefab.data_or_default(entity_index);
            prefab_data.mesh = Some(mesh);
            if let Some((material_id, material)) =
                material_index.and_then(|index| gltf.materials().nth(index).map(|m| (index, m)))
            {
                if !material_set.materials.contains_key(&material_id) {
                    material_set.materials.insert(
                        material_id,
                        load_material(&material, buffers, source.clone(), name)?,
                    );
                }
                prefab_data.material_id = Some(material_id);
            }
            // if we have a skin we need to track the mesh entities
            if let Some(ref mut skin) = skin {
                skin.mesh_indices.push(entity_index);
            }
        } else if graphics.len() > 1 {
            // if we have multiple primitives,
            // we need to add each primitive as a child entity to the node
            for (mesh, material_index, bounds) in graphics {
                let mesh_entity = prefab.add(Some(entity_index), None);
                let prefab_data = prefab.data_or_default(mesh_entity);
                prefab_data.transform = Some(Transform::default());
                prefab_data.mesh = Some(mesh);
                if let Some((material_id, material)) =
                    material_index.and_then(|index| gltf.materials().nth(index).map(|m| (index, m)))
                {
                    if !material_set.materials.contains_key(&material_id) {
                        material_set.materials.insert(
                            material_id,
                            load_material(&material, buffers, source.clone(), name)?,
                        );
                    }
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
