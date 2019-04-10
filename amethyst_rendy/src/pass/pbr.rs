use super::util;
use crate::{
    batch::{BatchData, BatchPrimitives, GroupIterator},
    camera::{ActiveCamera, Camera},
    hidden::{Hidden, HiddenPropagate},
    light::Light,
    mtl::{Material, MaterialDefaults},
    pod,
    resources::{AmbientColor, Tint},
    skinning::{JointTransforms, PosNormTangTexJoint},
    types::{Mesh, Texture},
};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    ecs::{Join, Read, ReadExpect, ReadStorage, Resources, SystemData},
    transform::GlobalTransform,
};
use derivative::Derivative;
use fnv::FnvHashMap;
use glsl_layout::*;
use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{PrepareResult, RenderGroup, RenderGroupDesc, SetLayout},
        BufferAccess, GraphContext, ImageAccess, NodeBuffer, NodeImage,
    },
    hal::{self, adapter::PhysicalDevice, device::Device, pso, Backend},
    mesh::{AsVertex, PosNormTangTex},
    resource::{DescriptorSet, DescriptorSetLayout, Escape, Handle as RendyHandle},
    shader::Shader,
};
use shred_derive::SystemData;
use smallvec::{smallvec, SmallVec};
use std::io::Write;

macro_rules! set_layout {
    ($factory:expr, $($times:literal $ty:ident $flags:ident),*) => {
        $factory.create_descriptor_set_layout(set_layout(
            std::iter::empty()
                $(.chain(std::iter::once(($times, pso::DescriptorType::$ty, pso::ShaderStageFlags::$flags))))*
        ).bindings)?.into()
    }
}

/// Draw mesh without lighting
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawPbrDesc {
    skinning: bool,
    transparency: Option<(pso::ColorBlendDesc, Option<pso::DepthStencilDesc>)>,
}

impl DrawPbrDesc {
    /// Create instance of `DrawPbr` pass
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
        color: pso::ColorBlendDesc,
        depth: Option<pso::DepthStencilDesc>,
    ) -> Self {
        self.transparency = Some((color, depth));
        self
    }
}

const MAX_POINT_LIGHTS: usize = 128;
const MAX_DIR_LIGHTS: usize = 16;
const MAX_SPOT_LIGHTS: usize = 128;

impl<B: Backend> RenderGroupDesc<B, Resources> for DrawPbrDesc {
    fn buffers(&self) -> Vec<BufferAccess> {
        vec![]
    }
    fn images(&self) -> Vec<ImageAccess> {
        vec![]
    }
    fn depth(&self) -> bool {
        true
    }
    fn colors(&self) -> usize {
        1
    }

    fn build<'a>(
        self,
        _ctx: &mut GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &Resources,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, Resources>>, failure::Error> {
        log::trace!("Loading shader module '{:#?}'", *super::BASIC_VERTEX);
        let shader_vertex_basic = unsafe { super::BASIC_VERTEX.module(factory).unwrap() };
        log::trace!("Loading shader module '{:#?}'", *super::PBR_FRAGMENT);
        let shader_fragment = unsafe { super::PBR_FRAGMENT.module(factory).unwrap() };

        let shader_vertex_skinned = if self.skinning {
            log::trace!("Loading shader module '{:#?}'", *super::SKINNED_VERTEX);
            Some(unsafe { super::SKINNED_VERTEX.module(factory).unwrap() })
        } else {
            None
        };

        let shader_set_basic =
            util::simple_shader_set(&shader_vertex_basic, Some(&shader_fragment));
        let shader_set_skinned = if self.skinning {
            Some(util::simple_shader_set(
                shader_vertex_skinned.as_ref().unwrap(),
                Some(&shader_fragment),
            ))
        } else {
            None
        };

        let set_layouts = DrawPbrLayouts {
            environment: set_layout! {factory, 1 UniformBuffer VERTEX, 4 UniformBuffer FRAGMENT},
            material: set_layout! {factory, 1 UniformBuffer FRAGMENT, 7 CombinedImageSampler FRAGMENT},
            skinning: set_layout! {factory, 1 StorageBuffer VERTEX},
        };

        let pipeline_layout = unsafe {
            factory.device().create_pipeline_layout(
                set_layouts.iter_raw(),
                None as Option<(pso::ShaderStageFlags, std::ops::Range<u32>)>,
            )
        }?;

        let mut vbos = Vec::new();
        let mut attributes = Vec::new();
        util::push_vertex_desc(
            PosNormTangTex::VERTEX.gfx_vertex_input_desc(0),
            &mut vbos,
            &mut attributes,
        );
        util::push_vertex_desc(
            pod::VertexArgs::VERTEX.gfx_vertex_input_desc(1),
            &mut vbos,
            &mut attributes,
        );

        let rect = pso::Rect {
            x: 0,
            y: 0,
            w: framebuffer_width as i16,
            h: framebuffer_height as i16,
        };

        let targets = if let Some((color, _)) = self.transparency {
            vec![color]
        } else {
            vec![pso::ColorBlendDesc(
                pso::ColorMask::ALL,
                pso::BlendState::ALPHA,
            )]
        };

        let multisampling = None;

        let mut descs = Vec::with_capacity(2);

        let parent_flags = if self.skinning {
            pso::PipelineCreationFlags::ALLOW_DERIVATIVES
        } else {
            pso::PipelineCreationFlags::empty()
        };

        let input_assembler = pso::InputAssemblerDesc {
            primitive: hal::Primitive::TriangleList,
            primitive_restart: pso::PrimitiveRestart::Disabled,
        };

        let depth_stencil = pso::DepthStencilDesc {
            depth: pso::DepthTest::On {
                fun: pso::Comparison::Less,
                write: true,
            },
            depth_bounds: false,
            stencil: pso::StencilTest::Off,
        };

        descs.push(pso::GraphicsPipelineDesc {
            shaders: shader_set_basic,
            rasterizer: pso::Rasterizer::FILL,
            vertex_buffers: vbos,
            attributes,
            input_assembler: input_assembler.clone(),
            blender: pso::BlendDesc {
                logic_op: None,
                targets: targets.clone(),
            },
            depth_stencil,
            multisampling: multisampling.clone(),
            baked_states: pso::BakedStates {
                viewport: Some(pso::Viewport {
                    rect,
                    depth: 0.0..1.0,
                }),
                scissor: Some(rect),
                blend_color: None,
                depth_bounds: None,
            },
            layout: &pipeline_layout,
            subpass,
            flags: parent_flags,
            parent: pso::BasePipeline::None,
        });

        if self.skinning {
            let mut vbos_skinned = Vec::new();
            let mut attributes_skinned = Vec::new();
            util::push_vertex_desc(
                PosNormTangTexJoint::VERTEX.gfx_vertex_input_desc(0),
                &mut vbos_skinned,
                &mut attributes_skinned,
            );
            util::push_vertex_desc(
                pod::SkinnedVertexArgs::VERTEX.gfx_vertex_input_desc(1),
                &mut vbos_skinned,
                &mut attributes_skinned,
            );

            descs.push(pso::GraphicsPipelineDesc {
                shaders: shader_set_skinned.unwrap(),
                rasterizer: pso::Rasterizer::FILL,
                vertex_buffers: vbos_skinned,
                attributes: attributes_skinned,
                input_assembler,
                blender: pso::BlendDesc {
                    logic_op: None,
                    targets,
                },
                depth_stencil,
                multisampling,
                baked_states: pso::BakedStates {
                    viewport: Some(pso::Viewport {
                        rect,
                        depth: 0.0..1.0,
                    }),
                    scissor: Some(rect),
                    blend_color: None,
                    depth_bounds: None,
                },
                layout: &pipeline_layout,
                subpass,
                flags: pso::PipelineCreationFlags::empty(),
                parent: pso::BasePipeline::Index(0),
            });
        }

        let mut pipelines = unsafe { factory.device().create_graphics_pipelines(descs, None) };
        let pipeline_basic = pipelines.remove(0)?;
        let pipeline_skinned = if self.skinning {
            Some(pipelines.remove(0)?)
        } else {
            None
        };

        unsafe {
            factory.destroy_shader_module(shader_vertex_basic);
            factory.destroy_shader_module(shader_fragment);
            shader_vertex_skinned.map(|m| factory.destroy_shader_module(m));
        }
        let limits = factory.physical().limits();
        Ok(Box::new(DrawPbr::<B> {
            pipeline_basic,
            pipeline_skinned,
            pipeline_layout,
            set_layouts,
            per_image: Vec::with_capacity(4),
            materials_data: Default::default(),
            ubo_offset_align: limits.min_uniform_buffer_offset_alignment,
        }))
    }
}

#[derive(Debug)]
struct DrawPbrLayouts<B: Backend> {
    environment: RendyHandle<DescriptorSetLayout<B>>,
    material: RendyHandle<DescriptorSetLayout<B>>,
    skinning: RendyHandle<DescriptorSetLayout<B>>,
}

impl<B: Backend> DrawPbrLayouts<B> {
    pub fn iter_raw(&self) -> impl Iterator<Item = &B::DescriptorSetLayout> {
        use std::iter::once;
        once(self.environment.raw())
            .chain(once(self.material.raw()))
            .chain(once(self.skinning.raw()))
    }
}

#[derive(Debug)]
pub struct DrawPbr<B: Backend> {
    pipeline_basic: B::GraphicsPipeline,
    pipeline_skinned: Option<B::GraphicsPipeline>,
    pipeline_layout: B::PipelineLayout,
    set_layouts: DrawPbrLayouts<B>,
    per_image: Vec<PerImage<B>>,
    materials_data: FnvHashMap<u32, MaterialData<B>>,
    ubo_offset_align: u64,
}

#[derive(Derivative, Debug)]
#[derivative(Default(bound = ""))]
struct PerImage<B: Backend> {
    environment_buffer: Option<Escape<rendy::resource::Buffer<B>>>,
    models_buffer: Option<Escape<rendy::resource::Buffer<B>>>,
    skinned_models_buffer: Option<Escape<rendy::resource::Buffer<B>>>,
    joints_buffer: Option<Escape<rendy::resource::Buffer<B>>>,
    material_buffer: Option<Escape<rendy::resource::Buffer<B>>>,
    environment_set: Option<Escape<DescriptorSet<B>>>,
    skinning_set: Option<Escape<DescriptorSet<B>>>,
}

#[derive(Debug, Derivative)]
#[derivative(Default(bound = ""))]
struct MaterialData<B: Backend> {
    // usually given material will have just one mesh
    static_batches: SmallVec<[StaticBatchData; 1]>,
    skinned_batches: SmallVec<[SkinnedBatchData; 1]>,
    desc_set: SmallVec<[Escape<DescriptorSet<B>>; 3]>,
}

type StaticBatchData = BatchData<u32, SmallVec<[pod::VertexArgs; 4]>>;
type SkinnedBatchData = BatchData<u32, SmallVec<[pod::SkinnedVertexArgs; 4]>>;

struct BatchStatic<B: Backend>(std::marker::PhantomData<B>);
struct BatchSkinned<B: Backend>(std::marker::PhantomData<B>);

impl<B: Backend> BatchPrimitives for BatchStatic<B> {
    type Shell = MaterialData<B>;
    type Batch = StaticBatchData;

    fn wrap_batch(batch: Self::Batch) -> Self::Shell {
        MaterialData {
            static_batches: smallvec![batch],
            ..Default::default()
        }
    }
    fn push(shell: &mut Self::Shell, batch: Self::Batch) {
        shell.static_batches.push(batch);
    }
    fn batches_mut(shell: &mut Self::Shell) -> &mut [Self::Batch] {
        &mut shell.static_batches
    }
}

impl<B: Backend> BatchPrimitives for BatchSkinned<B> {
    type Shell = MaterialData<B>;
    type Batch = SkinnedBatchData;

    fn wrap_batch(batch: Self::Batch) -> Self::Shell {
        MaterialData {
            skinned_batches: smallvec![batch],
            ..Default::default()
        }
    }
    fn push(shell: &mut Self::Shell, batch: Self::Batch) {
        shell.skinned_batches.push(batch);
    }
    fn batches_mut(shell: &mut Self::Shell) -> &mut [Self::Batch] {
        &mut shell.skinned_batches
    }
}

impl<B: Backend> DrawPbr<B> {
    #[inline]
    fn texture_descriptor<'a>(
        handle: &Handle<Texture<B>>,
        fallback: &Handle<Texture<B>>,
        storage: &'a AssetStorage<Texture<B>>,
    ) -> pso::Descriptor<'a, B> {
        let Texture(texture) = storage
            .get(handle)
            .or_else(|| storage.get(fallback))
            .unwrap();
        pso::Descriptor::CombinedImageSampler(
            texture.view().raw(),
            hal::image::Layout::ShaderReadOnlyOptimal,
            texture.sampler().raw(),
        )
    }

    #[inline]
    fn desc_write<'a>(
        set: &'a B::DescriptorSet,
        binding: u32,
        descriptor: pso::Descriptor<'a, B>,
    ) -> pso::DescriptorSetWrite<'a, B, Option<pso::Descriptor<'a, B>>> {
        pso::DescriptorSetWrite {
            set,
            binding,
            array_offset: 0,
            descriptors: Some(descriptor),
        }
    }
}

#[derive(SystemData)]
struct PbrPassData<'a, B: Backend> {
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
    joints: ReadStorage<'a, JointTransforms>,
    lights: ReadStorage<'a, Light>,
    tints: ReadStorage<'a, Tint>,
}

impl<B: Backend> RenderGroup<B, Resources> for DrawPbr<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        resources: &Resources,
    ) -> PrepareResult {
        let PbrPassData {
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
            joints,
            lights,
            tints,
            ..
        } = PbrPassData::<B>::fetch(resources);

        let set_layouts = &self.set_layouts;

        // ensure resources for this image are available
        let this_image = {
            while self.per_image.len() <= index {
                self.per_image.push(PerImage::default());
            }
            &mut self.per_image[index]
        };

        let (camera_position, projview) =
            util::prepare_camera(&active_camera, &cameras, &global_transforms);

        // Prepare environment
        {
            let align = self.ubo_offset_align;
            let projview_size = util::align_size::<pod::ViewArgs>(align, 1);
            let env_buf_size = util::align_size::<pod::Environment>(align, 1);
            let plight_buf_size = util::align_size::<pod::PointLight>(align, MAX_POINT_LIGHTS);
            let dlight_buf_size = util::align_size::<pod::DirectionalLight>(align, MAX_DIR_LIGHTS);
            let slight_buf_size = util::align_size::<pod::SpotLight>(align, MAX_SPOT_LIGHTS);

            let projview_range = Some(0)..Some(projview_size);
            let env_range = util::next_range_opt(&projview_range, env_buf_size);
            let plight_range = util::next_range_opt(&env_range, plight_buf_size);
            let dlight_range = util::next_range_opt(&plight_range, dlight_buf_size);
            let slight_range = util::next_range_opt(&dlight_range, slight_buf_size);

            if util::ensure_buffer(
                &factory,
                &mut this_image.environment_buffer,
                hal::buffer::Usage::UNIFORM,
                rendy::memory::Dynamic,
                slight_range.end.unwrap(),
            )
            .unwrap()
            {
                let buffer = this_image.environment_buffer.as_ref().unwrap().raw();
                let env_set = this_image
                    .environment_set
                    .get_or_insert_with(|| {
                        factory
                            .create_descriptor_set(set_layouts.environment.clone())
                            .unwrap()
                    })
                    .raw();

                let desc_projview = pso::Descriptor::Buffer(buffer, projview_range.clone());
                let desc_env = pso::Descriptor::Buffer(buffer, env_range.clone());
                let desc_plight = pso::Descriptor::Buffer(buffer, plight_range.clone());
                let desc_dlight = pso::Descriptor::Buffer(buffer, dlight_range.clone());
                let desc_slight = pso::Descriptor::Buffer(buffer, slight_range.clone());

                unsafe {
                    factory.write_descriptor_sets(vec![
                        Self::desc_write(env_set, 0, desc_projview),
                        Self::desc_write(env_set, 1, desc_env),
                        Self::desc_write(env_set, 2, desc_plight),
                        Self::desc_write(env_set, 3, desc_dlight),
                        Self::desc_write(env_set, 4, desc_slight),
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
            data.static_batches.clear();
            data.skinned_batches.clear();
        }

        let material_data_ref = &mut self.materials_data;
        let mut num_static_objects = 0;
        let mut num_skinned_objects = 0;

        (
            &materials,
            &meshes,
            &global_transforms,
            tints.maybe(),
            !&joints,
            !&hiddens,
            !&hiddens_prop,
        )
            .join()
            .map(|(material, mesh, transform, tint, _, _, _)| {
                (
                    (material.id(), mesh.id()),
                    pod::VertexArgs::from_object_data(transform, tint),
                )
            })
            .for_each_group(|(mat_id, mesh_id), batch_data| {
                if mesh_storage.contains_id(mesh_id) {
                    num_static_objects += batch_data.len();
                    BatchStatic::insert_batch(material_data_ref.entry(mat_id), mesh_id, batch_data);
                }
            });

        let joints_buffer_contents = if self.pipeline_skinned.is_some() {
            let mut joints_buffer: Vec<[[f32; 4]; 4]> = Vec::with_capacity(1024);
            let mut joints_offset_map: FnvHashMap<u32, u32> = Default::default();
            (
                &materials,
                &meshes,
                &global_transforms,
                tints.maybe(),
                &joints,
                !&hiddens,
                !&hiddens_prop,
            )
                .join()
                .map(|(material, mesh, transform, tint, joints, _, _)| {
                    let offset = joints_offset_map
                        .entry(joints.skin.id())
                        .or_insert_with(|| {
                            let len = joints_buffer.len();
                            joints_buffer.extend(
                                joints
                                    .matrices
                                    .iter()
                                    .map(|m| -> [[f32; 4]; 4] { (*m).into() }),
                            );
                            len as u32
                        });
                    (
                        (material.id(), mesh.id()),
                        pod::SkinnedVertexArgs::from_object_data(transform, tint, *offset),
                    )
                })
                .for_each_group(|(mat_id, mesh_id), batch_data| {
                    if mesh_storage.contains_id(mesh_id) {
                        num_skinned_objects += batch_data.len();
                        BatchSkinned::insert_batch(
                            material_data_ref.entry(mat_id),
                            mesh_id,
                            batch_data,
                        );
                    }
                });
            Some(joints_buffer)
        } else {
            None
        };

        self.materials_data
            .retain(|_, data| data.static_batches.len() > 0 || data.skinned_batches.len() > 0);

        let mut vertex_args: Vec<pod::VertexArgs> = Vec::with_capacity(num_static_objects);
        let mut skinned_vertex_args: Vec<pod::SkinnedVertexArgs> =
            Vec::with_capacity(num_skinned_objects);

        vertex_args.extend(
            self.materials_data
                .iter()
                .flat_map(|(_, mat)| mat.static_batches.iter().flat_map(|b| &b.collection)),
        );
        skinned_vertex_args.extend(
            self.materials_data
                .iter()
                .flat_map(|(_, mat)| mat.skinned_batches.iter().flat_map(|b| &b.collection)),
        );

        for (_, mat) in &mut self.materials_data {
            while mat.desc_set.len() <= index {
                mat.desc_set.push(
                    factory
                        .create_descriptor_set(set_layouts.material.clone())
                        .unwrap(),
                );
            }
        }

        let num_materials = self.materials_data.len();
        let material_step = util::align_size::<pod::Material>(self.ubo_offset_align, 1);
        let mut material_buffer_data: Vec<u8> = vec![0; num_materials * material_step as usize];

        util::ensure_buffer(
            &factory,
            &mut this_image.material_buffer,
            hal::buffer::Usage::UNIFORM,
            rendy::memory::Dynamic,
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
            let desc_material = pso::Descriptor::Buffer(
                this_image.material_buffer.as_mut().unwrap().raw(),
                Some(offset)..Some(offset + material_step),
            );
            let desc_albedo = Self::texture_descriptor(&mat.albedo, &def.albedo, storage);
            let desc_emission = Self::texture_descriptor(&mat.emission, &def.emission, storage);
            let desc_normal = Self::texture_descriptor(&mat.normal, &def.normal, storage);
            let desc_metallic = Self::texture_descriptor(&mat.metallic, &def.metallic, storage);
            let desc_roughness = Self::texture_descriptor(&mat.roughness, &def.roughness, storage);
            let desc_ao =
                Self::texture_descriptor(&mat.ambient_occlusion, &def.ambient_occlusion, storage);
            let desc_caveat = Self::texture_descriptor(&mat.caveat, &def.caveat, storage);

            unsafe {
                factory.write_descriptor_sets(vec![
                    Self::desc_write(set, 0, desc_material),
                    Self::desc_write(set, 1, desc_albedo),
                    Self::desc_write(set, 2, desc_emission),
                    Self::desc_write(set, 3, desc_normal),
                    Self::desc_write(set, 4, desc_metallic),
                    Self::desc_write(set, 5, desc_roughness),
                    Self::desc_write(set, 6, desc_ao),
                    Self::desc_write(set, 7, desc_caveat),
                ]);
            }
        }

        if let Some(joints_buffer_contents) = joints_buffer_contents.as_ref() {
            util::ensure_buffer(
                &factory,
                &mut this_image.skinned_models_buffer,
                hal::buffer::Usage::VERTEX,
                rendy::memory::Dynamic,
                (skinned_vertex_args.len() * std::mem::size_of::<pod::SkinnedVertexArgs>()) as _,
            )
            .unwrap();

            let size = joints_buffer_contents.len() * std::mem::size_of::<[[f32; 4]; 4]>();
            util::ensure_buffer(
                &factory,
                &mut this_image.joints_buffer,
                hal::buffer::Usage::STORAGE,
                rendy::memory::Dynamic,
                size as _,
            )
            .unwrap();

            let set = this_image
                .skinning_set
                .get_or_insert_with(|| {
                    factory
                        .create_descriptor_set(set_layouts.skinning.clone())
                        .unwrap()
                })
                .raw();
            if let Some(buffer) = this_image.joints_buffer.as_ref().map(|b| b.raw()) {
                let descriptor = pso::Descriptor::Buffer(buffer, Some(0)..Some(size as _));
                unsafe {
                    factory.write_descriptor_sets(Some(Self::desc_write(set, 0, descriptor)));
                }
            }
        }

        util::ensure_buffer(
            &factory,
            &mut this_image.models_buffer,
            hal::buffer::Usage::VERTEX,
            rendy::memory::Dynamic,
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

        if let Some(mut buffer) = this_image.skinned_models_buffer.as_mut() {
            unsafe {
                factory
                    .upload_visible_buffer(&mut buffer, 0, &skinned_vertex_args)
                    .unwrap();
            }
        }

        if let (Some(mut buffer), Some(data)) = (
            this_image.joints_buffer.as_mut(),
            joints_buffer_contents.as_ref(),
        ) {
            unsafe {
                factory.upload_visible_buffer(&mut buffer, 0, data).unwrap();
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

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        resources: &Resources,
    ) {
        let this_image = &self.per_image[index];
        encoder.bind_graphics_pipeline(&self.pipeline_basic);

        if let Some(environment_set) = this_image.environment_set.as_ref() {
            let PbrPassData { mesh_storage, .. } = PbrPassData::<B>::fetch(resources);

            encoder.bind_graphics_descriptor_sets(
                &self.pipeline_layout,
                0,
                Some(environment_set.raw()),
                std::iter::empty(),
            );

            let mut instances_drawn = 0;

            for (_, material) in &self.materials_data {
                encoder.bind_graphics_descriptor_sets(
                    &self.pipeline_layout,
                    1,
                    Some(material.desc_set[index].raw()),
                    std::iter::empty(),
                );

                for batch in &material.static_batches {
                    // This invariant should always be verified before inserting batches in prepare
                    debug_assert!(mesh_storage.contains_id(batch.key));
                    let Mesh(mesh) = unsafe { mesh_storage.get_by_id_unchecked(batch.key) };
                    mesh.bind(&[PosNormTangTex::VERTEX], &mut encoder).unwrap();
                    encoder.bind_vertex_buffers(
                        1,
                        Some((
                            this_image.models_buffer.as_ref().unwrap().raw(),
                            instances_drawn * std::mem::size_of::<pod::VertexArgs>() as u64,
                        )),
                    );
                    encoder.draw(0..mesh.len(), 0..batch.collection.len() as _);
                    instances_drawn += batch.collection.len() as u64;
                }
            }

            if let Some(pipeline_skinned) = self.pipeline_skinned.as_ref() {
                instances_drawn = 0;
                encoder.bind_graphics_pipeline(pipeline_skinned);

                encoder.bind_graphics_descriptor_sets(
                    &self.pipeline_layout,
                    2,
                    Some(this_image.skinning_set.as_ref().unwrap().raw()),
                    std::iter::empty(),
                );

                for (_, material) in &self.materials_data {
                    encoder.bind_graphics_descriptor_sets(
                        &self.pipeline_layout,
                        1,
                        Some(material.desc_set[index].raw()),
                        std::iter::empty(),
                    );

                    for batch in &material.skinned_batches {
                        // This invariant should always be verified before inserting batches in prepare
                        debug_assert!(mesh_storage.contains_id(batch.key));
                        let Mesh(mesh) = unsafe { mesh_storage.get_by_id_unchecked(batch.key) };
                        mesh.bind(&[PosNormTangTexJoint::VERTEX], &mut encoder)
                            .unwrap();

                        if let Some(buffer) =
                            this_image.skinned_models_buffer.as_ref().map(|b| b.raw())
                        {
                            encoder.bind_vertex_buffers(
                                1,
                                Some((
                                    buffer,
                                    instances_drawn
                                        * std::mem::size_of::<pod::SkinnedVertexArgs>() as u64,
                                )),
                            );
                        }
                        encoder.draw(0..mesh.len(), 0..batch.collection.len() as _);
                        instances_drawn += batch.collection.len() as u64;
                    }
                }
            }
        }
    }

    fn dispose(mut self: Box<Self>, factory: &mut Factory<B>, _aux: &Resources) {
        unsafe {
            factory
                .device()
                .destroy_graphics_pipeline(self.pipeline_basic);
            self.pipeline_skinned.take().map(|pipeline| {
                factory.device().destroy_graphics_pipeline(pipeline);
            });
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
            drop(self.set_layouts);
        }
    }
}

fn set_layout(
    bindings: impl IntoIterator<Item = (u32, pso::DescriptorType, pso::ShaderStageFlags)>,
) -> SetLayout {
    SetLayout {
        bindings: bindings
            .into_iter()
            .flat_map(|(times, ty, stage_flags)| (0..times).map(move |_| (ty, stage_flags)))
            .enumerate()
            .map(
                |(binding, (ty, stage_flags))| pso::DescriptorSetLayoutBinding {
                    binding: binding as u32,
                    ty,
                    count: 1,
                    stage_flags,
                    immutable_samplers: false,
                },
            )
            .collect(),
    }
}
