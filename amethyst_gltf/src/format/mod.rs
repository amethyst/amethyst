//! GLTF format

use std::{collections::HashMap, error::Error as StdError, fmt, sync::Arc};

use gltf::{self, Gltf};
use log::debug;

use crate::{
    animation::AnimationHierarchyPrefab,
    assets::{
        Error as AssetError, Format, FormatValue, Prefab, Result as AssetResult, ResultExt, Source,
    },
    core::{
        nalgebra::{Quaternion, Unit},
        transform::Transform,
    },
};

use super::*;

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

/// Format errors
#[derive(Debug)]
pub enum GltfError {
    /// Importer failed to load the json file
    GltfImporterError(self::importer::Error),

    /// GLTF have no default scene and the number of scenes is not 1
    InvalidSceneGltf(usize),

    /// GLTF primitive missing positions
    MissingPositions,

    /// GLTF animation channel missing input
    MissingInputs,

    /// GLTF animation channel missing output
    MissingOutputs,

    /// External file failed loading
    Asset(AssetError),

    /// Not implemented yet
    NotImplemented,
}

impl StdError for GltfError {
    fn description(&self) -> &str {
        use self::GltfError::*;
        match *self {
            GltfImporterError(_) => "Gltf import error",
            InvalidSceneGltf(_) => "Gltf has no default scene, and the number of scenes is not 1",
            MissingPositions => "Primitive missing positions",
            MissingInputs => "Channel missing inputs",
            MissingOutputs => "Channel missing outputs",
            Asset(_) => "File loading error",
            NotImplemented => "Not implemented",
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            GltfError::GltfImporterError(ref err) => Some(err),
            GltfError::Asset(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for GltfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::error::Error;

        use self::GltfError::*;

        match *self {
            GltfImporterError(ref err) => {
                write!(f, "{}: {}", self.description(), err.description())
            }
            Asset(ref err) => write!(f, "{}: {}", self.description(), err.description()),
            InvalidSceneGltf(size) => write!(f, "{}: {}", self.description(), size),
            MissingPositions | NotImplemented | MissingInputs | MissingOutputs => {
                write!(f, "{}", self.description())
            }
        }
    }
}

impl From<self::importer::Error> for GltfError {
    fn from(err: self::importer::Error) -> Self {
        GltfError::GltfImporterError(err)
    }
}

impl From<AssetError> for GltfError {
    fn from(err: AssetError) -> Self {
        GltfError::Asset(err)
    }
}

impl Format<Prefab<GltfPrefab>> for GltfSceneFormat {
    const NAME: &'static str = "GLTFScene";

    type Options = GltfSceneOptions;

    fn import(
        &self,
        name: String,
        source: Arc<dyn Source>,
        options: GltfSceneOptions,
        _create_reload: bool,
    ) -> AssetResult<FormatValue<Prefab<GltfPrefab>>> {
        Ok(FormatValue::data(
            load_gltf(source, &name, options).chain_err(|| "Failed to import gltf scene")?,
        ))
    }
}

fn load_gltf(
    source: Arc<dyn Source>,
    name: &str,
    options: GltfSceneOptions,
) -> Result<Prefab<GltfPrefab>, GltfError> {
    debug!("Loading GLTF scene {}", name);
    import(source.clone(), name)
        .map_err(GltfError::GltfImporterError)
        .and_then(|(gltf, buffers)| load_data(&gltf, &buffers, &options, source, name))
}

fn load_data(
    gltf: &Gltf,
    buffers: &Buffers,
    options: &GltfSceneOptions,
    source: Arc<dyn Source>,
    name: &str,
) -> Result<Prefab<GltfPrefab>, GltfError> {
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

fn get_scene_index(gltf: &Gltf, options: &GltfSceneOptions) -> Result<usize, GltfError> {
    let num_scenes = gltf.scenes().len();
    match (options.scene_index, gltf.default_scene()) {
        (Some(index), _) if index >= num_scenes => Err(GltfError::InvalidSceneGltf(num_scenes)),
        (Some(index), _) => Ok(index),
        (None, Some(scene)) => Ok(scene.index()),
        (None, _) if num_scenes > 1 => Err(GltfError::InvalidSceneGltf(num_scenes)),
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
) -> Result<(), GltfError> {
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
) -> Result<(), GltfError> {
    node_map.insert(node.index(), entity_index);

    // Load node name.
    if let Some(name) = node.name() {
        prefab.data_or_default(entity_index).name = Some(Named::new(name.to_string()));
    }

    // Load transformation data, default will be identity
    let (translation, rotation, scale) = node.transform().decomposed();
    let mut local_transform = Transform::default();
    *local_transform.translation_mut() = translation.into();
    // gltf quat format: [x, y, z, w], argument order expected by our quaternion: (w, x, y, z)
    *local_transform.rotation_mut() = Unit::new_normalize(Quaternion::new(
        rotation[3],
        rotation[0],
        rotation[1],
        rotation[2],
    ));
    *local_transform.scale_mut() = scale.into();
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
