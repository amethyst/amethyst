use super::util;
use crate::{
    camera::{ActiveCamera, Camera},
    hidden::{Hidden, HiddenPropagate},
    light::Light,
    mtl::{Material, MaterialDefaults},
    pod,
    resources::{AmbientColor, Tint},
    skinning::{JointIds, JointWeights},
    types::{Mesh, Texture},
};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    ecs::{Join, Read, ReadExpect, ReadStorage, Resources, SystemData},
    transform::GlobalTransform,
};
use fnv::FnvHashMap;
use glsl_layout::*;
use rendy::{
    command::{QueueId, RenderPassEncoder},
    descriptor::{DescriptorSet, DescriptorSetLayout},
    factory::Factory,
    graph::{
        render::{
            Layout, PrepareResult, SetLayout, SimpleGraphicsPipeline, SimpleGraphicsPipelineDesc,
        },
        GraphContext, NodeBuffer, NodeImage,
    },
    hal::{
        device::Device,
        format::Format,
        pso::{
            BlendState, ColorBlendDesc, ColorMask, DepthStencilDesc, Descriptor,
            DescriptorSetLayoutBinding, DescriptorSetWrite, DescriptorType, ElemStride, Element,
            EntryPoint, GraphicsShaderSet, InstanceRate, ShaderStageFlags, Specialization,
        },
        Backend,
    },
    mesh::{AsVertex, PosNormTangTex},
    resource::set::{DescriptorSet, DescriptorSetLayout},
    shader::Shader,
};
use shred_derive::SystemData;
use smallvec::{smallvec, SmallVec};
use std::io::Write;

/// Draw mesh without lighting
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawPbmDesc {
    skinning: bool,
    transparency: Option<(ColorBlendDesc, Option<DepthStencilDesc>)>,
}

impl DrawPbmDesc {
    /// Create instance of `DrawPbm` pass
    pub fn new() -> Self {
        Default::default()
    }

    /// Enable vertex skinning
    pub fn with_vertex_skinning(mut self) -> Self {
        self.skinning = true;
        self
    }

    /// Enable transparency
    pub fn with_transparency(
        mut self,
        color: ColorBlendDesc,
        depth: Option<DepthStencilDesc>,
    ) -> Self {
        self.transparency = Some((color, depth));
        self
    }
}

const MAX_POINT_LIGHTS: usize = 128;
const MAX_DIR_LIGHTS: usize = 16;
const MAX_SPOT_LIGHTS: usize = 128;

impl<B: Backend> SimpleGraphicsPipelineDesc<B, Resources> for DrawPbmDesc {
    type Pipeline = DrawPbm<B>;

    fn vertices(&self) -> Vec<(Vec<Element<Format>>, ElemStride, InstanceRate)> {
        let mut verts = vec![
            PosNormTangTex::VERTEX.gfx_vertex_input_desc(0),
            pod::VertexArgs::VERTEX.gfx_vertex_input_desc(1),
        ];
        if self.skinning {
            verts.push(JointWeights::VERTEX.gfx_vertex_input_desc(0));
            verts.push(JointIds::VERTEX.gfx_vertex_input_desc(0));
        }
        verts
    }

    fn layout(&self) -> Layout {
        let mut sets = Vec::with_capacity(4);
        // Set 0 - vertex args
        sets.push(SetLayout {
            bindings: vec![DescriptorSetLayoutBinding {
                binding: 0,
                ty: DescriptorType::UniformBuffer,
                count: 1,
                stage_flags: ShaderStageFlags::GRAPHICS,
                immutable_samplers: false,
            }],
        });
        // Set 1 - material
        let mut bindings = Vec::with_capacity(8);
        bindings.push(DescriptorSetLayoutBinding {
            binding: 0,
            ty: DescriptorType::UniformBuffer,
            count: 1,
            stage_flags: ShaderStageFlags::FRAGMENT,
            immutable_samplers: false,
        });
        for i in 1..8 {
            bindings.push(DescriptorSetLayoutBinding {
                binding: i,
                ty: DescriptorType::CombinedImageSampler,
                count: 1,
                stage_flags: ShaderStageFlags::FRAGMENT,
                immutable_samplers: false,
            });
        }
        sets.push(SetLayout { bindings });
        // Set 2 - environment
        let mut bindings = Vec::with_capacity(4);
        for i in 0..4 {
            bindings.push(DescriptorSetLayoutBinding {
                binding: i,
                ty: DescriptorType::UniformBuffer,
                count: 1,
                stage_flags: ShaderStageFlags::FRAGMENT,
                immutable_samplers: false,
            })
        }
        sets.push(SetLayout { bindings });

        if self.skinning {
            // Set 3 - skinning
            let skinning_layout = SetLayout {
                bindings: vec![DescriptorSetLayoutBinding {
                    binding: 0,
                    ty: DescriptorType::UniformBuffer,
                    count: 1,
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                }],
            };
            sets.push(skinning_layout);
        }

        Layout {
            sets,
            push_constants: Vec::new(),
        }
    }

    fn load_shader_set<'a>(
        &self,
        storage: &'a mut Vec<B::ShaderModule>,
        factory: &mut Factory<B>,
        _aux: &Resources,
    ) -> GraphicsShaderSet<'a, B> {
        storage.clear();

        log::trace!("Loading shader module '{:#?}'", *super::BASIC_VERTEX);
        storage.push(super::BASIC_VERTEX.module(factory).unwrap());
        log::trace!("Loading shader module '{:#?}'", *super::PBM_FRAGMENT);
        storage.push(super::PBM_FRAGMENT.module(factory).unwrap());

        if self.skinning {
            log::trace!("Loading shader module '{:#?}'", *super::SKINNED_VERTEX);
            storage.push(super::SKINNED_VERTEX.module(factory).unwrap());
        };

        GraphicsShaderSet {
            vertex: EntryPoint {
                entry: "main",
                module: &storage[0],
                specialization: Specialization::default(),
            },
            fragment: Some(EntryPoint {
                entry: "main",
                module: &storage[1],
                specialization: Specialization::default(),
            }),
            hull: None,
            domain: None,
            geometry: None,
        }
    }

    fn colors(&self) -> Vec<ColorBlendDesc> {
        if let Some((color, _)) = self.transparency {
            vec![color]
        } else {
            vec![ColorBlendDesc(ColorMask::ALL, BlendState::ALPHA)]
        }
    }

    fn build<'a>(
        self,
        _ctx: &mut GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _resources: &Resources,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
        set_layouts: &[DescriptorSetLayout<B>],
    ) -> Result<DrawPbm<B>, failure::Error> {
        let ubo_offset_align = {
            use rendy::hal::PhysicalDevice;
            factory
                .physical()
                .limits()
                .min_uniform_buffer_offset_alignment
        };

        let env_buffer = factory
            .create_buffer(
                16,
                std::mem::size_of::<pod::Environment>() as _,
                rendy::resource::buffer::UniformBuffer,
            )
            .unwrap();
        let plight_buffer = factory
            .create_buffer(
                16,
                std::mem::size_of::<pod::PointLight>() as _,
                rendy::resource::buffer::UniformBuffer,
            )
            .unwrap();
        let dlight_buffer = factory
            .create_buffer(
                16,
                std::mem::size_of::<pod::DirectionalLight>() as _,
                rendy::resource::buffer::UniformBuffer,
            )
            .unwrap();
        let slight_buffer = factory
            .create_buffer(
                16,
                std::mem::size_of::<pod::SpotLight>() as _,
                rendy::resource::buffer::UniformBuffer,
            )
            .unwrap();

        let environment_set = unsafe {
            let set = factory.create_descriptor_set(&set_layouts[2]).unwrap();
            factory.write_descriptor_sets(vec![
                DescriptorSetWrite {
                    set: set.raw(),
                    binding: 0,
                    array_offset: 0,
                    descriptors: Some(Descriptor::Buffer(env_buffer.raw(), None..None)),
                },
                DescriptorSetWrite {
                    set: set.raw(),
                    binding: 1,
                    array_offset: 0,
                    descriptors: Some(Descriptor::Buffer(plight_buffer.raw(), None..None)),
                },
                DescriptorSetWrite {
                    set: set.raw(),
                    binding: 2,
                    array_offset: 0,
                    descriptors: Some(Descriptor::Buffer(dlight_buffer.raw(), None..None)),
                },
                DescriptorSetWrite {
                    set: set.raw(),
                    binding: 3,
                    array_offset: 0,
                    descriptors: Some(Descriptor::Buffer(slight_buffer.raw(), None..None)),
                },
            ]);
            set
        };

        Ok(DrawPbm {
            skinning: self.skinning,
            env_buffer,
            plight_buffer,
            dlight_buffer,
            slight_buffer,
            object_buffer: Vec::new(),
            material_buffer: Vec::new(),
            material_data: Vec::new(),
            object_data: Vec::new(),
            environment_set,
            ubo_offset_align,
        })
    }
}

#[derive(Debug)]
pub struct DrawPbm<B: Backend> {
    skinning: bool,
    env_buffer: rendy::resource::Buffer<B>,
    plight_buffer: rendy::resource::Buffer<B>,
    dlight_buffer: rendy::resource::Buffer<B>,
    slight_buffer: rendy::resource::Buffer<B>,
    object_buffer: Vec<Option<rendy::resource::Buffer<B>>>,
    material_buffer: Vec<Option<rendy::resource::Buffer<B>>>,
    material_data: Vec<MaterialData<B>>,
    object_data: Vec<ObjectData<B>>,
    environment_set: DescriptorSet<B>,
    ubo_offset_align: u64,
}

impl<B: Backend> DrawPbm<B> {
    #[inline(always)]
    fn insert_batch(
        materials_data: &mut FnvHashMap<u32, MaterialData<B>>,
        (mat_id, mesh_id): (u32, u32),
        instance_data: impl IntoIterator<Item = pod::VertexArgs>,
        mesh_storage: &AssetStorage<Mesh<B>>,
    ) -> usize {
        if !mesh_storage.contains_id(mesh_id) {
            return 0;
        }

        let mut inserted = 0;
        use std::collections::hash_map::Entry;
        match materials_data.entry(mat_id) {
            Entry::Occupied(mut e) => {
                let mat = e.get_mut();

                // scan for the same mesh to try to combine batches.
                // Scanning up to next 8 slots to limit complexity.
                if let Some(batch) = mat
                    .batches
                    .iter_mut()
                    .take(8)
                    .find(|b| b.mesh_id == mesh_id)
                {
                    let old_len = batch.vertex_args.len();
                    batch.vertex_args.extend(instance_data);
                    inserted += batch.vertex_args.len() - old_len;
                } else {
                    let vertex_args: SmallVec<[pod::VertexArgs; 4]> =
                        instance_data.into_iter().collect();
                    inserted += vertex_args.len();
                    mat.batches.push(InstancedBatchData {
                        mesh_id,
                        vertex_args,
                    });
                }
            }
            Entry::Vacant(e) => {
                let vertex_args: SmallVec<[pod::VertexArgs; 4]> =
                    instance_data.into_iter().collect();
                inserted += vertex_args.len();
                e.insert(MaterialData {
                    batches: smallvec![InstancedBatchData {
                        mesh_id,
                        vertex_args
                    }],
                    desc_set: SmallVec::new(),
                });
            }
        };
        inserted
    }
}

#[derive(Debug)]
struct PerImage<B: Backend> {
    environment_buffer: Option<rendy::resource::Buffer<B>>,
    models_buffer: Option<rendy::resource::Buffer<B>>,
    material_buffer: Option<rendy::resource::Buffer<B>>,
    environment_set: Option<DescriptorSet<B>>,
    objects_set: Option<DescriptorSet<B>>,
}

impl<B: Backend> PerImage<B> {
    fn new() -> Self {
        Self {
            environment_buffer: None,
            models_buffer: None,
            material_buffer: None,
            environment_set: None,
            objects_set: None,
        }
    }
}

#[derive(Debug)]
struct MaterialData<B: Backend> {
    bit_set: BitSet,
    desc_set: Option<DescriptorSet<B>>,
    handle: Handle<Material<B>>,
}

#[derive(Debug)]
struct ObjectData<B: Backend> {
    desc_set: DescriptorSet<B>,
}

impl<B: Backend> DrawPbm<B> {
    fn texture_descriptor<'a>(
        handle: &Handle<Texture<B>>,
        fallback: &Handle<Texture<B>>,
        storage: &'a AssetStorage<Texture<B>>,
    ) -> Option<Descriptor<'a, B>> {
        storage
            .get(handle)
            .or_else(|| storage.get(fallback))
            .map(|Texture(texture)| {
                Descriptor::CombinedImageSampler(
                    texture.image_view.raw(),
                    rendy::hal::image::Layout::ShaderReadOnlyOptimal,
                    texture.sampler.raw(),
                )
            })
    }
}

#[derive(SystemData)]
struct PbmPassData<'a, B: Backend> {
    ambient_color: Option<Read<'a, AmbientColor>>,
    active_camera: Option<Read<'a, ActiveCamera>>,
    cameras: ReadStorage<'a, Camera>,
    mesh_storage: Read<'a, AssetStorage<Mesh<B>>>,
    texture_storage: Read<'a, AssetStorage<Texture<B>>>,
    material_storage: Read<'a, AssetStorage<Material<B>>>,
    material_defaults: ReadExpect<'a, MaterialDefaults<B>>,
    // visibility: Option<Read<'a, Visibility>>,
    hiddens: ReadStorage<'a, Hidden>,
    hiddens_prop: ReadStorage<'a, HiddenPropagate>,
    meshes: ReadStorage<'a, Handle<Mesh<B>>>,
    materials: ReadStorage<'a, Handle<Material<B>>>,
    global_transforms: ReadStorage<'a, GlobalTransform>,
    // joints: ReadStorage<'a, JointTransforms>,
    lights: ReadStorage<'a, Light>,
    tints: ReadStorage<'a, Tint>,
}

impl<B: Backend> SimpleGraphicsPipeline<B, Resources> for DrawPbm<B> {
    type Desc = DrawPbmDesc;

    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        set_layouts: &[DescriptorSetLayout<B>],
        index: usize,
        resources: &Resources,
    ) -> PrepareResult {
        let PbmPassData {
            ambient_color,
            active_camera,
            cameras,
            mesh_storage,
            texture_storage,
            material_storage,
            material_defaults,
            // visibility,
            hiddens,
            hiddens_prop,
            meshes,
            materials,
            global_transforms,
            // joints,
            lights,
            tints,
            ..
        } = PbmPassData::<B>::fetch(resources);

        let defcam = Camera::standard_2d();
        let identity = GlobalTransform::default();
        let camera = active_camera
            .and_then(|ac| {
                cameras.get(ac.entity).map(|camera| {
                    (
                        camera,
                        global_transforms.get(ac.entity).unwrap_or(&identity),
                    )
                })
            })
            .unwrap_or_else(|| {
                (&cameras, &global_transforms)
                    .join()
                    .next()
                    .unwrap_or((&defcam, &identity))
            });

        let point_lights: Vec<_> = (&lights, &global_transforms)
            .join()
            .filter_map(|(light, transform)| {
                if let Light::Point(ref light) = *light {
                    Some(
                        pod::PointLight {
                            position: pod_vec(transform.0.column(3).xyz()),
                            color: pod_srgb(light.color),
                            intensity: light.intensity,
                        }
                        .std140(),
                    )
                } else {
                    None
                }
            })
            .collect();

        let dir_lights: Vec<_> = lights
            .join()
            .filter_map(|light| {
                if let Light::Directional(ref light) = *light {
                    Some(
                        pod::DirectionalLight {
                            color: pod_srgb(light.color),
                            direction: pod_vec(light.direction),
                        }
                        .std140(),
                    )
                } else {
                    None
                }
            })
            .collect();

        let spot_lights: Vec<_> = (&lights, &global_transforms)
            .join()
            .filter_map(|(light, transform)| {
                if let Light::Spot(ref light) = *light {
                    Some(
                        pod::SpotLight {
                            position: pod_vec(transform.0.column(3).xyz()),
                            color: pod_srgb(light.color),
                            direction: pod_vec(light.direction),
                            angle: light.angle.cos(),
                            intensity: light.intensity,
                            range: light.range,
                            smoothness: light.smoothness,
                        }
                        .std140(),
                    )
                } else {
                    None
                }
            })
            .collect();

        let pod = pod::Environment {
            ambient_color: [0.0, 0.0, 0.0].into(), // TODO: ambient
            camera_position: pod_vec((camera.1).0.column(3).xyz()),
            point_light_count: point_lights.len() as _,
            directional_light_count: dir_lights.len() as _,
            spot_light_count: spot_lights.len() as _,
        }
        .std140();

        unsafe {
            factory
                .upload_visible_buffer(&mut self.env_buffer, 0, &[pod])
                .unwrap();
            factory
                .upload_visible_buffer(&mut self.plight_buffer, 0, &point_lights)
                .unwrap();
            factory
                .upload_visible_buffer(&mut self.dlight_buffer, 0, &dir_lights)
                .unwrap();
            factory
                .upload_visible_buffer(&mut self.slight_buffer, 0, &spot_lights)
                .unwrap();
        }

        factory.destroy_descriptor_sets(self.material_data.drain(..).filter_map(|d| d.desc_set));
        self.material_data.clear();
        let mut total_objects = 0;

        {
            let mut materials_hash: FnvHashMap<Handle<Material<B>>, u32> = Default::default();

            let joinable = (
                &entities,
                &materials,
                &meshes,
                &global_transforms,
                !&hiddens,
            );

            for (entity, material, _, _, _) in joinable.join() {
                use std::collections::hash_map::Entry;
                total_objects += 1;
                match materials_hash.entry(material.clone()) {
                    Entry::Occupied(e) => {
                        let mat = &mut self.material_data[*e.get() as usize];
                        mat.bit_set.add(entity.id());
                    }
                    Entry::Vacant(e) => {
                        e.insert(self.material_data.len() as u32);
                        let mut bit_set = BitSet::new();
                        bit_set.add(entity.id());
                        self.material_data.push(MaterialData {
                            bit_set,
                            desc_set: None,
                            handle: material.clone(),
                        });
                    }
                }
            }
        }

        let material_step = align_size::<pod::Material>(self.ubo_offset_align);
        let mut material_buffer_data: Vec<u8> =
            vec![0; self.material_data.len() * material_step as usize];

        while self.material_buffer.len() <= index {
            self.material_buffer.push(None);
        }

        ensure_buffer(
            &factory,
            &mut self.material_buffer[index],
            rendy::resource::buffer::UniformBuffer,
            self.material_data.len() as u64 * material_step,
        )
        .unwrap();

        for (i, material) in self.material_data.iter_mut().enumerate() {
            use super::util::TextureOffset;

            let def = &material_defaults.0;
            let mat = material_storage.get(&material.handle).unwrap_or(def);
            let storage = &texture_storage;

        let (camera_position, projview) =
            util::prepare_camera(&active_camera, &cameras, &global_transforms);

        // Prepare environment
        {
            let align = self.ubo_offset_align;
            let env_buf_size = util::align_size::<pod::Environment>(align, 1);
            let plight_buf_size = util::align_size::<pod::PointLight>(align, MAX_POINT_LIGHTS);
            let dlight_buf_size = util::align_size::<pod::DirectionalLight>(align, MAX_DIR_LIGHTS);
            let slight_buf_size = util::align_size::<pod::SpotLight>(align, MAX_SPOT_LIGHTS);
            let projview_size = util::align_size::<pod::ViewArgs>(align, 1);

            let env_range = Some(0)..Some(env_buf_size);
            let plight_range = util::next_range_opt(&env_range, plight_buf_size);
            let dlight_range = util::next_range_opt(&plight_range, dlight_buf_size);
            let slight_range = util::next_range_opt(&dlight_range, slight_buf_size);
            let projview_range = util::next_range_opt(&slight_range, projview_size);

            if util::ensure_buffer(
                &factory,
                &mut this_image.environment_buffer,
                rendy::resource::buffer::UniformBuffer,
                slight_range.end.unwrap(),
            )
            .unwrap()
            {
                let buffer = this_image.environment_buffer.as_ref().unwrap().raw();
                let env_set = this_image
                    .environment_set
                    .get_or_insert_with(|| factory.create_descriptor_set(&set_layouts[2]).unwrap())
                    .raw();

                let obj_set = this_image
                    .objects_set
                    .get_or_insert_with(|| factory.create_descriptor_set(&set_layouts[0]).unwrap())
                    .raw();

            unsafe {
                if let Ok(set) = factory.create_descriptor_set(&set_layouts[1]) {
                    factory.write_descriptor_sets(vec![
                        DescriptorSetWrite {
                            set: set.raw(),
                            binding: 0,
                            array_offset: 0,
                            descriptors: Some(Descriptor::Buffer(buffer, env_range.clone())),
                        },
                        DescriptorSetWrite {
                            set: set.raw(),
                            binding: 1,
                            array_offset: 0,
                            descriptors: Some(Descriptor::Buffer(buffer, plight_range.clone())),
                        },
                        DescriptorSetWrite {
                            set: set.raw(),
                            binding: 2,
                            array_offset: 0,
                            descriptors: Some(Descriptor::Buffer(buffer, dlight_range.clone())),
                        },
                        DescriptorSetWrite {
                            set: set.raw(),
                            binding: 3,
                            array_offset: 0,
                            descriptors: Some(Descriptor::Buffer(buffer, slight_range.clone())),
                        },
                        DescriptorSetWrite {
                            set: set.raw(),
                            binding: 4,
                            array_offset: 0,
                            descriptors: Self::texture_descriptor(
                                &mat.metallic,
                                &def.metallic,
                                storage,
                            ),
                        },
                        DescriptorSetWrite {
                            set: set.raw(),
                            binding: 5,
                            array_offset: 0,
                            descriptors: Self::texture_descriptor(
                                &mat.roughness,
                                &def.roughness,
                                storage,
                            ),
                        },
                        DescriptorSetWrite {
                            set: set.raw(),
                            binding: 6,
                            array_offset: 0,
                            descriptors: Self::texture_descriptor(
                                &mat.ambient_occlusion,
                                &def.ambient_occlusion,
                                storage,
                            ),
                        },
                        DescriptorSetWrite {
                            set: set.raw(),
                            binding: 7,
                            array_offset: 0,
                            descriptors: Some(Descriptor::Buffer(buffer, projview_range.clone())),
                        },
                    ]);
                }
            }

            let (point_lights, dir_lights, spot_lights) = util::collect_lights(
                &lights,
                &global_transforms,
                MAX_POINT_LIGHTS,
                MAX_DIR_LIGHTS,
                MAX_SPOT_LIGHTS,
            );

            let ambient_color = ambient_color.map_or([0.0, 0.0, 0.0].into(), |c| {
                let (r, g, b, _) = c.0.into_components();
                [r, g, b].into()
            });

            let pod = pod::Environment {
                ambient_color,
                camera_position,
                point_light_count: point_lights.len() as _,
                directional_light_count: dir_lights.len() as _,
                spot_light_count: spot_lights.len() as _,
            }
            .std140();

            unsafe {
                let buffer = this_image.environment_buffer.as_mut().unwrap();
                factory
                    .upload_visible_buffer(buffer, env_range.start.unwrap(), &[pod])
                    .unwrap();
                if point_lights.len() > 0 {
                    factory
                        .upload_visible_buffer(buffer, plight_range.start.unwrap(), &point_lights)
                        .unwrap();
                }
                if dir_lights.len() > 0 {
                    factory
                        .upload_visible_buffer(buffer, dlight_range.start.unwrap(), &dir_lights)
                        .unwrap();
                }
                if spot_lights.len() > 0 {
                    factory
                        .upload_visible_buffer(buffer, slight_range.start.unwrap(), &spot_lights)
                        .unwrap();
                }

                factory
                    .upload_visible_buffer(buffer, projview_range.start.unwrap(), &[projview])
                    .unwrap();
            }
        }

        factory.destroy_descriptor_sets(self.object_data.drain(..).map(|d| d.desc_set));
        self.object_data.reserve(total_objects);

        // (material, mesh_id, instances)
        let mut block: Option<((u32, u32), Vec<pod::VertexArgs>)> = None;
        let mut total_objects = 0;

        for (material, mesh, transform, tint, _, _) in (
            &materials,
            &meshes,
            &global_transforms,
            tints.maybe(),
            !&hiddens,
            !&hiddens_prop,
        )
            .join()
        {
            let next_batch_id = (material.id(), mesh.id());

            match &mut block {
                slot @ None => {
                    let mut batch_data = Vec::with_capacity(32);
                    batch_data.push(pod::VertexArgs::from_object_data(transform, tint));
                    slot.replace((next_batch_id, batch_data));
                }
                Some((batch_id, batch_data)) if batch_id == &next_batch_id => {
                    batch_data.push(pod::VertexArgs::from_object_data(transform, tint));
                }
                Some((batch_id, batch_data)) => {
                    total_objects += Self::insert_batch(
                        &mut self.materials_data,
                        *batch_id,
                        batch_data.drain(..),
                        &mesh_storage,
                    );
                    batch_data.clear();
                    *batch_id = next_batch_id;
                    batch_data.push(pod::VertexArgs::from_object_data(transform, tint));
                }
            }
        }
        if let Some((batch_id, batch_data)) = block.take() {
            total_objects += Self::insert_batch(
                &mut self.materials_data,
                batch_id,
                batch_data,
                &mesh_storage,
            );
        }

        self.materials_data.retain(|_, data| data.batches.len() > 0);

        let mut vertex_args: Vec<pod::VertexArgs> = Vec::with_capacity(total_objects);
        vertex_args.extend(
            self.materials_data
                .iter()
                .flat_map(|(_, mat)| mat.batches.iter().flat_map(|b| &b.vertex_args)),
        );

        for (_, mat) in &mut self.materials_data {
            while mat.desc_set.len() <= index {
                mat.desc_set
                    .push(factory.create_descriptor_set(&set_layouts[1]).unwrap());
            }
        }

        let num_materials = self.materials_data.len();
        let material_step = util::align_size::<pod::Material>(self.ubo_offset_align, 1);
        let mut material_buffer_data: Vec<u8> = vec![0; num_materials * material_step as usize];

        util::ensure_buffer(
            &factory,
            &mut this_image.material_buffer,
            rendy::resource::buffer::UniformBuffer,
            num_materials as u64 * material_step,
        )
        .unwrap();

        for (i, (mat_id, data)) in self.materials_data.iter().enumerate() {
            let def = &material_defaults.0;
            let mat = material_storage.get_by_id(*mat_id).unwrap_or(def);
            let storage = &texture_storage;

            let pod = pod::Material::from_material(&mat).std140();

                unsafe {
                    let set = factory.create_descriptor_set(&set_layouts[0]).unwrap();
                    factory.write_descriptor_sets(vec![DescriptorSetWrite {
                        set: set.raw(),
                        binding: 0,
                        array_offset: 0,
                        descriptors: Some(Descriptor::Buffer(
                            this_image.material_buffer.as_mut().unwrap().raw(),
                            Some(offset)..Some(offset + material_step),
                        )),
                    },
                    DescriptorSetWrite {
                        set,
                        binding: 1,
                        array_offset: 0,
                        descriptors: Self::texture_descriptor(&mat.albedo, &def.albedo, storage),
                    },
                    DescriptorSetWrite {
                        set,
                        binding: 2,
                        array_offset: 0,
                        descriptors: Self::texture_descriptor(
                            &mat.emission,
                            &def.emission,
                            storage,
                        ),
                    },
                    DescriptorSetWrite {
                        set,
                        binding: 3,
                        array_offset: 0,
                        descriptors: Self::texture_descriptor(&mat.normal, &def.normal, storage),
                    },
                    DescriptorSetWrite {
                        set,
                        binding: 4,
                        array_offset: 0,
                        descriptors: Self::texture_descriptor(
                            &mat.metallic,
                            &def.metallic,
                            storage,
                        ),
                    },
                    DescriptorSetWrite {
                        set,
                        binding: 5,
                        array_offset: 0,
                        descriptors: Self::texture_descriptor(
                            &mat.roughness,
                            &def.roughness,
                            storage,
                        ),
                    },
                    DescriptorSetWrite {
                        set,
                        binding: 6,
                        array_offset: 0,
                        descriptors: Self::texture_descriptor(
                            &mat.ambient_occlusion,
                            &def.ambient_occlusion,
                            storage,
                        ),
                    },
                    DescriptorSetWrite {
                        set,
                        binding: 7,
                        array_offset: 0,
                        descriptors: Self::texture_descriptor(&mat.caveat, &def.caveat, storage),
                    },
                ]);
            }
        }

        util::ensure_buffer(
            &factory,
            &mut this_image.models_buffer,
            (rendy::hal::buffer::Usage::VERTEX, rendy::memory::Dynamic),
            (vertex_args.len() * std::mem::size_of::<pod::VertexArgs>()) as _,
        )
        .unwrap();

        if let Some(mut buffer) = this_image.material_buffer.as_mut() {
            unsafe {
                factory
                    .upload_visible_buffer(&mut buffer, 0, &material_buffer_data)
                    .unwrap();
            }
        }

        if let Some(mut buffer) = this_image.models_buffer.as_mut() {
            unsafe {
                factory
                    .upload_visible_buffer(&mut buffer, 0, &vertex_args)
                    .unwrap();
            }
        }

        // match visibility {
        //     None => {

        //         unimplemented!()
        //     }
        //     Some(ref visibility) => unimplemented!(),
        // }

        PrepareResult::DrawRecord
    }

    fn draw(
        &mut self,
        layout: &B::PipelineLayout,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        resources: &Resources,
    ) {
        encoder.bind_graphics_descriptor_sets(
            layout,
            2,
            Some(self.environment_set.raw()),
            std::iter::empty(),
        );

        let PbmPassData {
            mesh_storage,
            meshes,
            ..
        } = PbmPassData::<B>::fetch(resources);

        let unprotected_meshes = meshes.unprotected_storage();

        if let Some(objects_set) = this_image.objects_set.as_ref() {
            let PbmPassData { mesh_storage, .. } = PbmPassData::<B>::fetch(resources);

        for material in &self.material_data {
            if let Some(ref mat_set) = material.desc_set {
                encoder.bind_graphics_descriptor_sets(
                    layout,
                    1,
                    Some(mat_set.raw()),
                    std::iter::empty(),
                );
                for entity_id in &material.bit_set {
                    let handle = unsafe { unprotected_meshes.get(entity_id) };

            encoder.bind_graphics_descriptor_sets(
                layout,
                0,
                Some(objects_set.raw()),
                std::iter::empty(),
            );

                    if let Some(Mesh(mesh)) = mesh_storage.get(handle) {
                        encoder.bind_graphics_descriptor_sets(
                            layout,
                            0,
                            Some(obj_set.raw()),
                            std::iter::empty(),
                        );
                        mesh.bind(&[PosNormTangTex::VERTEX], &mut encoder).unwrap();
                        encoder.draw(0..mesh.len(), 0..1);
                    }
                }
            }
        }
        assert!(obj_data_iter.next().is_none());
    }

    fn dispose(mut self, factory: &mut Factory<B>, _aux: &Resources) {
        let all_sets = std::iter::once(self.environment_set)
            .chain(self.object_data.drain(..).map(|d| d.desc_set))
            .chain(self.material_data.drain(..).filter_map(|d| d.desc_set));
        factory.destroy_descriptor_sets(all_sets);
    }
}

fn pod_srgb(srgb: palette::Srgb) -> glsl_layout::vec3 {
    let (r, g, b) = srgb.into_components();
    [r, g, b].into()
}

fn pod_vec(vec: amethyst_core::math::Vector3<f32>) -> glsl_layout::vec3 {
    let arr: [f32; 3] = vec.into();
    arr.into()
}

fn align_size<T: AsStd140>(align: u64) -> u64
where
    T::Std140: Sized,
{
    let size = std::mem::size_of::<T::Std140>() as u64;
    ((size + align - 1) / align) * align
}

fn ensure_buffer<B: Backend>(
    factory: &Factory<B>,
    buffer: &mut Option<rendy::resource::Buffer<B>>,
    usage: impl rendy::resource::buffer::Usage,
    min_size: u64,
) -> Result<(), failure::Error> {
    if buffer.as_ref().map(|b| b.size()).unwrap_or(0) < min_size {
        let new_size = min_size.next_power_of_two();
        let new_buffer = factory.create_buffer(512, new_size, usage)?;
        *buffer = Some(new_buffer);
    }
    Ok(())
}

fn byte_size<T>(slice: &[T]) -> usize {
    slice.len() * std::mem::size_of::<T>()
}

mod pod {
    use super::super::util::TextureOffset;
    use glsl_layout::*;

    pub(crate) fn array_size<T: AsStd140>(elems: usize) -> usize
    where
        T::Std140: Sized,
    {
        std::mem::size_of::<T::Std140>() * elems
    }

    #[derive(Clone, Copy, Debug, AsStd140)]
    pub(crate) struct PointLight {
        pub position: vec3,
        pub color: vec3,
        pub intensity: float,
    }

    #[derive(Clone, Copy, Debug, AsStd140)]
    pub(crate) struct DirectionalLight {
        pub color: vec3,
        pub direction: vec3,
    }

    #[derive(Clone, Copy, Debug, AsStd140)]
    pub(crate) struct SpotLight {
        pub position: vec3,
        pub color: vec3,
        pub direction: vec3,
        pub angle: float,
        pub intensity: float,
        pub range: float,
        pub smoothness: float,
    }

    #[derive(Clone, Copy, Debug, AsStd140)]
    pub(crate) struct Environment {
        pub ambient_color: vec3,
        pub camera_position: vec3,
        pub point_light_count: int,
        pub directional_light_count: int,
        pub spot_light_count: int,
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &Resources) {}
}
