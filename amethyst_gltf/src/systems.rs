use std::collections::{HashMap, HashSet};

use animation::{Animation, AnimationHierarchy, AnimationSet, Joint, Sampler, Skin};
use assets::{AssetStorage, Handle, HotReloadStrategy, Loader};
use core::{ThreadPool, Time};
use core::cgmath::{EuclideanSpace, Matrix4, Point3, SquareMatrix};
use core::transform::*;
use fnv::FnvHashMap;
use hibitset::BitSet;
use renderer::{AnimatedComboMeshCreator, JointTransforms, Material, MaterialDefaults, Mesh,
               Texture};
use specs::{Entities, Entity, Fetch, FetchMut, Join, System, WriteStorage};

use {GltfMaterial, GltfPrimitive, GltfSceneAsset, GltfSkin};

/// A GLTF scene loader, will transform `Handle<GltfSceneAsset>` into full entity hierarchies.
///
/// Will also do the asset storage processing for `GltfSceneAsset`.
pub struct GltfSceneLoaderSystem {
    _dummy: (),
}

enum TextureHandleLocation {
    BaseColor,
    Metallic,
    Roughness,
    Emissive,
    Normal,
    Occlusion,
}

impl GltfSceneLoaderSystem {
    pub fn new() -> Self {
        Self { _dummy: () }
    }
}

impl<'a> System<'a> for GltfSceneLoaderSystem {
    type SystemData = (
        Entities<'a>,
        Fetch<'a, AssetStorage<Mesh>>,
        Fetch<'a, AssetStorage<Texture>>,
        Fetch<'a, AssetStorage<Animation>>,
        Fetch<'a, AssetStorage<Sampler>>,
        Fetch<'a, Loader>,
        Fetch<'a, MaterialDefaults>,
        Fetch<'a, Time>,
        Fetch<'a, ThreadPool>,
        Option<Fetch<'a, HotReloadStrategy>>,
        FetchMut<'a, AssetStorage<GltfSceneAsset>>,
        WriteStorage<'a, Handle<GltfSceneAsset>>,
        WriteStorage<'a, Handle<Mesh>>,
        WriteStorage<'a, LocalTransform>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Parent>,
        WriteStorage<'a, Material>,
        WriteStorage<'a, AnimationHierarchy>,
        WriteStorage<'a, AnimationSet>,
        WriteStorage<'a, Joint>,
        WriteStorage<'a, Skin>,
        WriteStorage<'a, JointTransforms>,
    );

    #[allow(unused)]
    fn run(&mut self, data: Self::SystemData) {
        use std::ops::Deref;

        let (
            entities,
            mesh_storage,
            texture_storage,
            animation_storage,
            sampler_storage,
            loader,
            material_defaults,
            time,
            pool,
            strategy,
            mut scene_storage,
            mut scenes,
            mut meshes,
            mut local_transforms,
            mut transforms,
            mut parents,
            mut materials,
            mut animation_hierarchies,
            mut animation_sets,
            mut joints,
            mut skins,
            mut joint_transforms,
        ) = data;

        let strategy = strategy.as_ref().map(Deref::deref);
        scene_storage.process(Into::into, time.frame_number(), &**pool, strategy);

        let mut deletes = vec![];

        // Scenes should really use FlaggedStorage
        for (entity, scene_handle) in (&*entities, &scenes).join() {
            if let Some(scene_asset) = scene_storage.get_mut(scene_handle) {
                // We need to track any new mesh/texture loads for caching purposes
                let mut mesh_handles = Vec::default();
                let mut texture_handles: Vec<
                    (usize, TextureHandleLocation, Handle<Texture>),
                > = Vec::default();
                let mut node_map = HashMap::default();
                let mut skin_links = Vec::default();

                // Use the default scene if set, otherwise use the first scene.
                // Note that the format will throw an error if the default scene is not set,
                // and there are more than one scene in the GLTF, so defaulting to scene 0 is safe
                let scene_index = scene_asset.default_scene.unwrap_or(0);
                let scene = &scene_asset.scenes[scene_index];

                // If we only have one root node in the scene, we load that node onto the attached
                // entity
                if scene.root_nodes.len() == 1 {
                    load_node(
                        scene.root_nodes[0],
                        &entity,
                        scene_asset,
                        &mut local_transforms,
                        &mut transforms,
                        &mut parents,
                        &entities,
                        &loader,
                        &mut meshes,
                        &mesh_storage,
                        &mut materials,
                        &texture_storage,
                        &*material_defaults,
                        &mut mesh_handles,
                        &mut texture_handles,
                        &mut node_map,
                        &mut skin_links,
                    );
                } else {
                    // If we have multiple root nodes in the scene, we need to create new entities
                    // for each root node and set their parent reference to the attached entity
                    for root_node_index in &scene.root_nodes {
                        let root_entity = entities.create();
                        parents.insert(root_entity, Parent { entity });
                        transforms.insert(root_entity, Transform::default());
                        load_node(
                            *root_node_index,
                            &root_entity,
                            scene_asset,
                            &mut local_transforms,
                            &mut transforms,
                            &mut parents,
                            &entities,
                            &loader,
                            &mut meshes,
                            &mesh_storage,
                            &mut materials,
                            &texture_storage,
                            &*material_defaults,
                            &mut mesh_handles,
                            &mut texture_handles,
                            &mut node_map,
                            &mut skin_links,
                        );
                    }
                }

                for (node_entity, skin_index, mesh_entities) in skin_links {
                    load_skin(
                        &scene_asset.skins[skin_index],
                        &node_entity,
                        &mut joints,
                        &mut skins,
                        &mut joint_transforms,
                        mesh_entities,
                        &node_map,
                    );
                }

                // process new mesh handles, cache them in the primitives
                for (node_index, primitive_index, handle) in mesh_handles {
                    scene_asset.nodes[node_index].primitives[primitive_index].handle = Some(handle);
                }

                // process new texture handles, cache them in the textures
                for (material_index, texture_index, handle) in texture_handles {
                    use self::TextureHandleLocation::*;
                    match texture_index {
                        BaseColor => {
                            scene_asset.materials[material_index].base_color.0.handle = Some(handle)
                        }
                        Metallic => {
                            scene_asset.materials[material_index].metallic.0.handle = Some(handle)
                        }
                        Roughness => {
                            scene_asset.materials[material_index].roughness.0.handle = Some(handle)
                        }
                        Normal => {
                            scene_asset.materials[material_index]
                                .normal
                                .as_mut()
                                .unwrap()
                                .0
                                .handle = Some(handle)
                        }
                        Occlusion => {
                            scene_asset.materials[material_index]
                                .occlusion
                                .as_mut()
                                .unwrap()
                                .0
                                .handle = Some(handle)
                        }
                        Emissive => {
                            scene_asset.materials[material_index].emissive.0.handle = Some(handle)
                        }
                        _ => unreachable!(),
                    }
                }

                // Load animations
                if scene_asset.options.load_animations && scene_asset.animations.len() > 0 {
                    // if handle doesn't exist, load animation data
                    let mut node_indices: HashSet<usize> = HashSet::default();
                    for animation in &mut scene_asset.animations {
                        trace!("Loading animation: {:?}", animation.nodes);
                        node_indices.extend(animation.nodes.iter());
                        if let None = animation.handle {
                            let samplers = animation
                                .samplers
                                .iter()
                                .cloned()
                                .map(|sampler| {
                                    loader.load_from_data(sampler, (), &*sampler_storage)
                                })
                                .collect::<Vec<_>>();
                            let sampler_map = samplers
                                .into_iter()
                                .enumerate()
                                .map(|(index, sampler)| (animation.nodes[index].clone(), sampler))
                                .collect::<Vec<_>>();
                            animation.handle = Some(loader.load_from_data(
                                Animation { nodes: sampler_map },
                                (),
                                &*animation_storage,
                            ));
                        }
                    }
                    let h = AnimationHierarchy {
                        nodes: node_indices
                            .into_iter()
                            .map(|node_index| {
                                (node_index, node_map.get(&node_index).cloned().unwrap())
                            })
                            .collect::<FnvHashMap<_, _>>(),
                    };
                    // create animation hierarchy
                    animation_hierarchies.insert(entity, h);
                    // create animation set
                    animation_sets.insert(
                        entity,
                        AnimationSet {
                            animations: scene_asset
                                .animations
                                .iter()
                                .filter_map(|a| a.handle.as_ref())
                                .cloned()
                                .collect::<Vec<_>>(),
                        },
                    );
                }
                deletes.push(entity);
            }
        }

        // FIXME: Use FlaggedStorage
        // For now we remove the scene handle after loading the scene
        for entity in deletes {
            scenes.remove(entity);
        }
    }
}

// Load a single node, attach all data to the given `node_entity`.
fn load_node(
    node_index: usize,
    node_entity: &Entity,
    scene_asset: &GltfSceneAsset,
    local_transforms: &mut WriteStorage<LocalTransform>,
    transforms: &mut WriteStorage<Transform>,
    parents: &mut WriteStorage<Parent>,
    entities: &Entities,
    loader: &Loader,
    meshes: &mut WriteStorage<Handle<Mesh>>,
    mesh_storage: &AssetStorage<Mesh>,
    materials: &mut WriteStorage<Material>,
    texture_storage: &AssetStorage<Texture>,
    material_defaults: &MaterialDefaults,
    mesh_handles: &mut Vec<(usize, usize, Handle<Mesh>)>,
    texture_handles: &mut Vec<(usize, TextureHandleLocation, Handle<Texture>)>,
    node_map: &mut HashMap<usize, Entity>,
    skin_links: &mut Vec<(Entity, usize, Vec<Entity>)>,
) {
    let node = &scene_asset.nodes[node_index];
    node_map.insert(node_index, node_entity.clone());

    // Load the node-to-parent transformation
    local_transforms.insert(*node_entity, node.local_transform.clone());

    // Load child entities
    for child_node_index in &node.children {
        let child_entity = entities.create();
        parents.insert(
            child_entity,
            Parent {
                entity: *node_entity,
            },
        );
        transforms.insert(child_entity, Transform::default());
        load_node(
            *child_node_index,
            &child_entity,
            scene_asset,
            local_transforms,
            transforms,
            parents,
            entities,
            loader,
            meshes,
            mesh_storage,
            materials,
            texture_storage,
            material_defaults,
            mesh_handles,
            texture_handles,
            node_map,
            skin_links,
        );
    }

    if node.primitives.len() > 0 {
        if scene_asset.options.move_to_origin {
            let mut min = Point3::origin();
            let mut max = Point3::origin();
            for primitive in &node.primitives {
                let pmin = &primitive.extents.start;
                let pmax = &primitive.extents.end;
                for i in 0..3 {
                    if pmin[i] < min[i] {
                        min[i] = pmin[i];
                    }
                    if pmax[i] > max[i] {
                        max[i] = pmax[i];
                    }
                }
            }
            let center = (min + max.to_vec()) / 2.;
            local_transforms
                .get_mut(*node_entity)
                .map(|t| t.translation -= center.to_vec());
            debug!("Moving node from: {:?} to origin", center);
        }
        let mut mesh_entities = vec![];
        // If we only have a single graphics primitive, we load it onto the nodes entity
        if node.primitives.len() == 1 {
            load_primitive(
                node_index,
                0,
                scene_asset,
                &node.primitives[0],
                node_entity,
                loader,
                meshes,
                mesh_storage,
                materials,
                texture_storage,
                material_defaults,
                mesh_handles,
                texture_handles,
            );
            mesh_entities.push(*node_entity);
        } else {
            // If we have multiple graphics primitives, we need to add child entities for each
            // primitive and load the graphics onto those
            for (primitive_index, primitive) in node.primitives.iter().enumerate() {
                let primitive_entity = entities.create();
                local_transforms.insert(primitive_entity, LocalTransform::default());
                transforms.insert(primitive_entity, Transform::default());
                parents.insert(
                    primitive_entity,
                    Parent {
                        entity: *node_entity,
                    },
                );
                load_primitive(
                    node_index,
                    primitive_index,
                    scene_asset,
                    primitive,
                    &primitive_entity,
                    loader,
                    meshes,
                    mesh_storage,
                    materials,
                    texture_storage,
                    material_defaults,
                    mesh_handles,
                    texture_handles,
                );
                mesh_entities.push(primitive_entity);
            }
        }

        if let Some(skin_index) = node.skin {
            skin_links.push((*node_entity, skin_index, mesh_entities));
        }
    }
}

// Load skin for a node
fn load_skin(
    skin: &GltfSkin,
    node_entity: &Entity,
    joints: &mut WriteStorage<Joint>,
    skins: &mut WriteStorage<Skin>,
    joint_transforms: &mut WriteStorage<JointTransforms>,
    mesh_entities: Vec<Entity>,
    node_map: &HashMap<usize, Entity>,
) {
    // Load joint node information
    for (skin_internal_index, joint_node_index) in skin.joints.iter().enumerate() {
        let joint_entity = node_map.get(joint_node_index).unwrap();
        joints.insert(
            *joint_entity,
            Joint {
                inverse_bind_matrix: Matrix4::from(skin.inverse_bind_matrices[skin_internal_index]),
                skin: *node_entity,
            },
        );
    }

    // Add joint transform holders for all mesh entities linked
    let mut mesh_bitset = BitSet::new();
    for mesh_entity in &mesh_entities {
        joint_transforms.insert(
            *mesh_entity,
            JointTransforms {
                skin: *node_entity,
                matrices: vec![Matrix4::identity().into(); skin.joints.len()],
            },
        );
        mesh_bitset.add(mesh_entity.id());
    }

    // Add skin to node with skin reference
    let joints = skin.joints
        .iter()
        .map(|i| *node_map.get(i).unwrap())
        .collect();

    let s = Skin {
        joints,
        meshes: mesh_bitset,
        bind_shape_matrix: Matrix4::identity(),
    };
    skins.insert(*node_entity, s);
}

// Load a single graphics primitive, attach all data to the given `entity`.
fn load_primitive(
    node_index: usize,
    primitive_index: usize,
    scene_asset: &GltfSceneAsset,
    primitive: &GltfPrimitive,
    entity: &Entity,
    loader: &Loader,
    meshes: &mut WriteStorage<Handle<Mesh>>,
    mesh_storage: &AssetStorage<Mesh>,
    materials: &mut WriteStorage<Material>,
    texture_storage: &AssetStorage<Texture>,
    material_defaults: &MaterialDefaults,
    mesh_handles: &mut Vec<(usize, usize, Handle<Mesh>)>,
    texture_handles: &mut Vec<(usize, TextureHandleLocation, Handle<Texture>)>,
) {
    let mesh = primitive.handle.as_ref().cloned().unwrap_or_else(|| {
        let mesh_creator = AnimatedComboMeshCreator::new(primitive.attributes.clone());
        let handle = loader.load_from_data(mesh_creator.into(), (), mesh_storage);
        mesh_handles.push((node_index, primitive_index, handle.clone()));
        handle
    });

    // Load material for the primitive
    let material = primitive.material
        .and_then(|index| scene_asset.materials.get(index).map(|m| (index, m)))
        .map(|(index, material)| load_material(
            index,
            material,
            loader,
            texture_storage,
            material_defaults,
            texture_handles,
        ))
        // If no material is defined, or the material is missing, use the default material
        .unwrap_or_else(|| material_defaults.0.clone());

    // Attach mesh to the entity
    meshes.insert(*entity, mesh);
    materials.insert(*entity, material);
}

// Load a material
fn load_material(
    material_index: usize,
    material: &GltfMaterial,
    loader: &Loader,
    texture_storage: &AssetStorage<Texture>,
    material_defaults: &MaterialDefaults,
    texture_handles: &mut Vec<(usize, TextureHandleLocation, Handle<Texture>)>,
) -> Material {
    use self::TextureHandleLocation::*;
    // TODO: base color factor
    // TODO: metallic/roughness factors
    // TODO: normal scale
    // TODO: emissive factor
    // TODO: alpha
    // TODO: double sided
    let albedo = material
        .base_color
        .0
        .handle
        .as_ref()
        .cloned()
        .unwrap_or_else(|| {
            let handle =
                loader.load_from_data(material.base_color.0.data.clone(), (), texture_storage);
            texture_handles.push((material_index, BaseColor, handle.clone()));
            handle
        });

    let metallic = material
        .metallic
        .0
        .handle
        .as_ref()
        .cloned()
        .unwrap_or_else(|| {
            let handle =
                loader.load_from_data(material.metallic.0.data.clone(), (), texture_storage);
            texture_handles.push((material_index, Metallic, handle.clone()));
            handle
        });

    let roughness = material
        .roughness
        .0
        .handle
        .as_ref()
        .cloned()
        .unwrap_or_else(|| {
            let handle =
                loader.load_from_data(material.roughness.0.data.clone(), (), texture_storage);
            texture_handles.push((material_index, Roughness, handle.clone()));
            handle
        });

    let normal = material.normal.as_ref().map(|&(ref normal, _)| {
        normal.handle.as_ref().cloned().unwrap_or_else(|| {
            let handle = loader.load_from_data(normal.data.clone(), (), texture_storage);
            texture_handles.push((material_index, Normal, handle.clone()));
            handle
        })
    });

    let ambient_occlusion = material.occlusion.as_ref().map(|&(ref occlusion, _)| {
        occlusion.handle.as_ref().cloned().unwrap_or_else(|| {
            let handle = loader.load_from_data(occlusion.data.clone(), (), texture_storage);
            texture_handles.push((material_index, Occlusion, handle.clone()));
            handle
        })
    });

    let emission = material
        .emissive
        .0
        .handle
        .as_ref()
        .cloned()
        .unwrap_or_else(|| {
            let handle =
                loader.load_from_data(material.emissive.0.data.clone(), (), texture_storage);
            texture_handles.push((material_index, Emissive, handle.clone()));
            handle
        });

    let mut mat = Material {
        albedo,
        emission,
        metallic,
        roughness,
        ..material_defaults.0.clone()
    };
    match normal {
        Some(n) => mat.normal = n,
        None => (),
    }
    match ambient_occlusion {
        Some(a) => mat.ambient_occlusion = a,
        None => (),
    }
    mat
}
