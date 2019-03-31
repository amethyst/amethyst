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
        _set_layouts: &[DescriptorSetLayout<B>],
    ) -> Result<DrawPbm<B>, failure::Error> {
        use rendy::hal::PhysicalDevice;
        let limits = factory.physical().limits();

        Ok(DrawPbm {
            skinning: self.skinning,
            per_image: Vec::with_capacity(4),
            materials_data: Default::default(),
            ubo_offset_align: limits.min_uniform_buffer_offset_alignment,
        })
    }
}

#[derive(Debug)]
pub struct DrawPbm<B: Backend> {
    skinning: bool,
    per_image: Vec<PerImage<B>>,
    materials_data: FnvHashMap<u32, MaterialData<B>>,
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
    // usually given material will have just one mesh
    batches: SmallVec<[InstancedBatchData; 1]>,
    desc_set: SmallVec<[DescriptorSet<B>; 3]>,
}

#[derive(Debug)]
struct InstancedBatchData {
    mesh_id: u32,
    vertex_args: SmallVec<[pod::VertexArgs; 4]>,
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

        // ensure resources for this image are available
        let this_image = {
            while self.per_image.len() <= index {
                self.per_image.push(PerImage::new());
            }
            &mut self.per_image[index]
        };

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
                    factory.write_descriptor_sets(vec![
                        DescriptorSetWrite {
                            set: env_set,
                            binding: 0,
                            array_offset: 0,
                            descriptors: Some(Descriptor::Buffer(buffer, env_range.clone())),
                        },
                        DescriptorSetWrite {
                            set: env_set,
                            binding: 1,
                            array_offset: 0,
                            descriptors: Some(Descriptor::Buffer(buffer, plight_range.clone())),
                        },
                        DescriptorSetWrite {
                            set: env_set,
                            binding: 2,
                            array_offset: 0,
                            descriptors: Some(Descriptor::Buffer(buffer, dlight_range.clone())),
                        },
                        DescriptorSetWrite {
                            set: env_set,
                            binding: 3,
                            array_offset: 0,
                            descriptors: Some(Descriptor::Buffer(buffer, slight_range.clone())),
                        },
                        DescriptorSetWrite {
                            set: obj_set,
                            binding: 0,
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

        // material setup
        for (_, data) in self.materials_data.iter_mut() {
            data.batches.clear();
        }

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

            let offset = material_step * i as u64;
            (&mut material_buffer_data[offset as usize..(offset + material_step) as usize])
                .write(as_bytes(&pod))
                .unwrap();

            let set = data.desc_set[index].raw();

            unsafe {
                factory.write_descriptor_sets(vec![
                    DescriptorSetWrite {
                        set,
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
        let this_image = &self.per_image[index];

        if let Some(objects_set) = this_image.objects_set.as_ref() {
            let PbmPassData { mesh_storage, .. } = PbmPassData::<B>::fetch(resources);

            encoder.bind_graphics_descriptor_sets(
                layout,
                2,
                Some(this_image.environment_set.as_ref().unwrap().raw()),
                std::iter::empty(),
            );

            encoder.bind_graphics_descriptor_sets(
                layout,
                0,
                Some(objects_set.raw()),
                std::iter::empty(),
            );

            let mut instances_drawn = 0;

            for (_, material) in &self.materials_data {
                encoder.bind_graphics_descriptor_sets(
                    layout,
                    1,
                    Some(material.desc_set[index].raw()),
                    std::iter::empty(),
                );

                for batch in &material.batches {
                    // This invariant should always be verified before inserting batches in prepare
                    debug_assert!(mesh_storage.contains_id(batch.mesh_id));
                    let Mesh(mesh) = unsafe { mesh_storage.get_by_id_unchecked(batch.mesh_id) };
                    mesh.bind(&[PosNormTangTex::VERTEX], &mut encoder).unwrap();
                    encoder.bind_vertex_buffers(
                        1,
                        Some((
                            this_image.models_buffer.as_ref().unwrap().raw(),
                            instances_drawn * std::mem::size_of::<pod::VertexArgs>() as u64,
                        )),
                    );
                    encoder.draw(0..mesh.len(), 0..batch.vertex_args.len() as _);
                    instances_drawn += batch.vertex_args.len() as u64;
                }
            }
        }
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &Resources) {}
}
