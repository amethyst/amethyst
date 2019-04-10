use crate::{
    batch::BatchPrimitives,
    camera::{ActiveCamera, Camera},
    hidden::{Hidden, HiddenPropagate},
    pass::util,
    pod::{self, IntoPod, SpriteArgs},
    sprite::{Sprite, SpriteCamera, SpriteRender, SpriteSheet, SpriteSheetHandle},
    sprite_visibility::SpriteVisibility,
    types::{Mesh, Texture},
    visibility::Visibility,
};

use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    ecs::{Join, Read, ReadStorage, Resources, SystemData},
    math::Vector4,
    transform::GlobalTransform,
};
use derivative::Derivative;
use fnv::FnvHashMap;
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
        adapter::PhysicalDevice,
        buffer::Usage as BufferUsage,
        device::Device,
        format::Format,
        pso::{
            BlendState, ColorBlendDesc, ColorMask, DepthStencilDesc, Descriptor,
            DescriptorSetLayoutBinding, DescriptorSetWrite, DescriptorType, ElemStride, Element,
            EntryPoint, GraphicsShaderSet, InstanceRate, ShaderStageFlags, Specialization,
        },
        Backend,
    },
    mesh::AsVertex,
    resource::{DescriptorSet, DescriptorSetLayout, Escape, Handle as RendyHandle},
    shader::Shader,
};
use smallvec::{smallvec, SmallVec};

/// Draw mesh without lighting
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawFlat2DDesc {
    transparency: Option<(ColorBlendDesc, Option<DepthStencilDesc>)>,
}

impl DrawFlat2DDesc {
    /// Create instance of `DrawFlat` pass
    pub fn new() -> Self {
        Default::default()
    }

    /// Enable transparency
    pub fn with_transparency(
        mut self,
        color: ColorBlendDesc,
        depth: Option<DepthStencilDesc>,
    ) -> Self {
        log::trace!("Transparency set: {:?}, {:?}", color, depth);
        self.transparency = Some((color, depth));
        self
    }
}

impl<B: Backend> SimpleGraphicsPipelineDesc<B, Resources> for DrawFlat2DDesc {
    type Pipeline = DrawFlat2D<B>;

    fn load_shader_set<'a>(
        &self,
        storage: &'a mut Vec<B::ShaderModule>,
        factory: &mut Factory<B>,
        _aux: &Resources,
    ) -> GraphicsShaderSet<'a, B> {
        storage.clear();

        log::trace!("Loading shader module '{:#?}'", *super::SPRITE_VERTEX);
        storage.push(unsafe { super::SPRITE_VERTEX.module(factory).unwrap() });

        log::trace!("Loading shader module '{:#?}'", *super::SPRITE_FRAGMENT);
        storage.push(unsafe { super::SPRITE_FRAGMENT.module(factory).unwrap() });

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
        log::trace!("colors()");
        if let Some((color, _)) = self.transparency {
            vec![color]
        } else {
            vec![ColorBlendDesc(ColorMask::ALL, BlendState::ALPHA)]
        }
    }

    fn depth_stencil(&self) -> Option<DepthStencilDesc> {
        log::trace!("depth_stencil()");
        if let Some((_, stencil)) = self.transparency {
            stencil
        } else {
            None
        }
    }

    fn vertices(&self) -> Vec<(Vec<Element<Format>>, ElemStride, InstanceRate)> {
        log::trace!("vertices()");
        vec![SpriteArgs::VERTEX.gfx_vertex_input_desc(0)]
    }

    fn layout(&self) -> Layout {
        log::trace!("layout()");
        Layout {
            sets: vec![
                SetLayout {
                    bindings: vec![DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: DescriptorType::UniformBuffer,
                        count: 1,
                        stage_flags: ShaderStageFlags::GRAPHICS,
                        immutable_samplers: false,
                    }],
                },
                SetLayout {
                    bindings: vec![DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: DescriptorType::SampledImage,
                        count: 32,
                        stage_flags: ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    }],
                },
            ],
            push_constants: vec![(ShaderStageFlags::FRAGMENT, 0..4)],
        }
    }

    fn build<'a>(
        self,
        ctx: &mut GraphContext<B>,
        factory: &mut Factory<B>,
        queue: QueueId,
        resource: &Resources,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
        set_layouts: &[RendyHandle<DescriptorSetLayout<B>>],
    ) -> Result<Self::Pipeline, failure::Error> {
        log::trace!("build()");

        let mut projview_buffer: Option<Escape<rendy::resource::Buffer<B>>> = None;
        let mut tex_buffer: Option<Escape<rendy::resource::Buffer<B>>> = None;
        let mut tex_id_buffer: Option<Escape<rendy::resource::Buffer<B>>> = None;

        let limits = factory.physical().limits();

        let projview_size =
            util::align_size::<pod::ViewArgs>(limits.min_uniform_buffer_offset_alignment, 1);
        let projview_set: Option<Escape<DescriptorSet<B>>>;

        util::ensure_buffer(
            &factory,
            &mut projview_buffer,
            BufferUsage::UNIFORM,
            rendy::memory::Dynamic,
            projview_size,
        )
        .unwrap();

        util::ensure_buffer(
            &factory,
            &mut tex_id_buffer,
            BufferUsage::UNIFORM,
            rendy::memory::Dynamic,
            4,
        )
        .unwrap();

        let projview_set = factory
            .create_descriptor_set(set_layouts[0].clone())
            .unwrap();

        let tex_set = factory
            .create_descriptor_set(set_layouts[1].clone())
            .unwrap();

        Ok(DrawFlat2D {
            projview_buffer,
            tex_id_buffer,
            projview_set: Some(projview_set),
            tex_set: Some(tex_set),
            ubo_offset_align: limits.min_uniform_buffer_offset_alignment,
            ..Default::default()
        })
    }
}

#[derive(Debug, Derivative)]
#[derivative(Default(bound = ""))]
pub struct DrawFlat2D<B: Backend> {
    projview_buffer: Option<Escape<rendy::resource::Buffer<B>>>,
    projview_set: Option<Escape<DescriptorSet<B>>>,
    vertex_buffers: Vec<Option<Escape<rendy::resource::Buffer<B>>>>,
    tex_id_buffer: Option<Escape<rendy::resource::Buffer<B>>>,
    tex_set: Option<Escape<DescriptorSet<B>>>,
    batches: FnvHashMap<u32, (Vec<pod::SpriteArgs>, u32)>,
    texture_map: FnvHashMap<u32, u32>,
    ubo_offset_align: u64,
}
impl<B: Backend> DrawFlat2D<B> {
    #[inline]
    fn desc_write<'a>(
        set: &'a B::DescriptorSet,
        binding: u32,
        array_offset: usize,
        descriptor: Descriptor<'a, B>,
    ) -> DescriptorSetWrite<'a, B, Option<Descriptor<'a, B>>> {
        DescriptorSetWrite {
            set,
            binding,
            array_offset,
            descriptors: Some(descriptor),
        }
    }
}
impl<B: Backend> SimpleGraphicsPipeline<B, Resources> for DrawFlat2D<B> {
    type Desc = DrawFlat2DDesc;

    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        set_layouts: &[RendyHandle<DescriptorSetLayout<B>>],
        _index: usize,
        resources: &Resources,
    ) -> PrepareResult {
        let (
            active_camera,
            sprite_camera,
            cameras,
            sprite_sheet_storage,
            tex_storage,
            visibilities,
            hiddens,
            hidden_props,
            sprite_renders,
            global_transforms,
            texture_handles,
        ) = <(
            Option<Read<'_, ActiveCamera>>,
            Option<Read<'_, SpriteCamera>>,
            ReadStorage<'_, Camera>,
            Read<'_, AssetStorage<SpriteSheet<B>>>,
            Read<'_, AssetStorage<Texture<B>>>,
            Option<Read<'_, SpriteVisibility>>,
            ReadStorage<'_, Hidden>,
            ReadStorage<'_, HiddenPropagate>,
            ReadStorage<'_, SpriteRender<B>>,
            ReadStorage<'_, GlobalTransform>,
            ReadStorage<'_, Handle<Texture<B>>>,
        ) as SystemData>::fetch(resources);

        let (camera_position, projview) = if sprite_camera.is_some() {
            util::prepare_camera(&sprite_camera, &cameras, &global_transforms)
        } else {
            util::prepare_camera(&active_camera, &cameras, &global_transforms)
        };

        let mut texture_constant_id: u32 = 0;

        match visibilities {
            None => {
                for (sprite_render, global, _, _) in (
                    &sprite_renders,
                    &global_transforms,
                    !&hiddens,
                    !&hidden_props,
                )
                    .join()
                {
                    log::trace!("Rendering a sprite");
                    let sprite_sheet = sprite_sheet_storage
                            .get(&sprite_render.sprite_sheet)
                            .expect(
                                "Unreachable: Existence of sprite sheet checked when collecting the sprites",
                            );
                    let tex_id = sprite_sheet.texture.id();
                    let sprite_data = &sprite_sheet.sprites[sprite_render.sprite_number];

                    let transform = &global.0;
                    let dir_x = transform.column(0) * sprite_data.width;
                    let dir_y = transform.column(1) * sprite_data.height;
                    let pos = transform
                        * Vector4::new(-sprite_data.offsets[0], -sprite_data.offsets[1], 0.0, 1.0);

                    let batch_data = SpriteArgs {
                        dir_x: dir_x.xy().into_pod(),
                        dir_y: dir_y.xy().into_pod(),
                        pos: pos.xy().into_pod(),
                        u_offset: [sprite_data.tex_coords.left, sprite_data.tex_coords.right]
                            .into(),
                        v_offset: [sprite_data.tex_coords.bottom, sprite_data.tex_coords.top]
                            .into(),
                        depth: pos.z,
                    };

                    if let Some(batch) = self.batches.get_mut(&tex_id) {
                        batch.0.push(batch_data);
                    } else {
                        let mut newbatch = Vec::with_capacity(1024);
                        newbatch.push(batch_data);
                        self.batches.insert(tex_id, (newbatch, texture_constant_id));
                        texture_constant_id += 1;
                    }
                }
            }
            Some(ref visibility) => {
                for (sprite_render, global, _) in (
                    &sprite_renders,
                    &global_transforms,
                    &visibility.visible_unordered,
                )
                    .join()
                {
                    log::trace!("WAT");
                    // self.batch.add_sprite(
                    //     &sprite_render,
                    //     Some(&global),
                    //     &sprite_sheet_storage,
                    //     &tex_storage,
                    // );
                }

                for entity in &visibility.visible_ordered {
                    //let screen = screens.contains(*entity);
                    if let Some(sprite_render) = sprite_renders.get(*entity) {
                        //self.batch.add_sprite(
                        //    &sprite_render,
                        //    global_transforms.get(*entity),
                        //    &sprite_sheet_storage,
                        //    &tex_storage,
                        //);
                    }
                }
            }
        }

        let mut desc_writes: SmallVec<[DescriptorSetWrite<'_, B, Option<Descriptor<'_, B>>>; 36]> =
            smallvec![];
        // Write the projview set.
        let limits = factory.physical().limits();
        let projview_size =
            util::align_size::<pod::ViewArgs>(limits.min_uniform_buffer_offset_alignment, 1);
        let desc_projview = Descriptor::Buffer(
            self.projview_buffer.as_ref().unwrap().raw(),
            Some(0)..Some(projview_size),
        );
        desc_writes.push(Self::desc_write(
            self.tex_set.as_ref().unwrap().raw(),
            0,
            0,
            desc_projview,
        ));

        let mut instances_written = 0;
        for (n, (tex_id, (batch_data, texture_constant_id))) in self.batches.iter().enumerate() {
            log::trace!("Iterating batch: {},{},{}", n, tex_id, texture_constant_id);
            if let Some(texture) = tex_storage.get_by_id(*tex_id) {
                desc_writes.push(Self::desc_write(
                    self.tex_set.as_ref().unwrap().raw(),
                    1,
                    n,
                    Descriptor::CombinedImageSampler(
                        texture.0.view().raw(),
                        rendy::hal::image::Layout::ShaderReadOnlyOptimal,
                        texture.0.sampler().raw(),
                    ),
                ));

                log::trace!("HIT 1");
                if let Some(mut buffer) = self.vertex_buffers.get_mut(*texture_constant_id as usize)
                {
                    util::ensure_buffer(
                        &factory,
                        buffer,
                        BufferUsage::UNIFORM,
                        rendy::memory::Dynamic,
                        4,
                    )
                    .unwrap();
                } else {
                    let mut buffer: Option<Escape<rendy::resource::Buffer<B>>> = None;
                    util::ensure_buffer(
                        &factory,
                        &mut buffer,
                        BufferUsage::UNIFORM,
                        rendy::memory::Dynamic,
                        4,
                    )
                    .unwrap();
                    self.vertex_buffers.push(buffer);
                }
                log::trace!("HIT 2");
                if let Some(mut buffer) = self.vertex_buffers.get_mut(*texture_constant_id as usize)
                {
                    if let Some(mut buffer) = buffer.as_mut() {
                        unsafe {
                            factory
                                .upload_visible_buffer(&mut buffer, 0, batch_data.as_slice())
                                .unwrap();

                            instances_written += 1;
                        }
                    } else {
                        log::warn!("Failed to get buffer as mut");
                        continue;
                    }
                } else {
                    log::warn!("Failed to get buffer from array");
                    continue;
                }
            } else {
                log::warn!("Failed to fetch texture");
            }
        }

        if instances_written > 1 {
            log::trace!("Rendering {} instances", instances_written);

            unsafe {
                factory.write_descriptor_sets(desc_writes);

                factory
                    .upload_visible_buffer(self.projview_buffer.as_mut().unwrap(), 0, &[projview])
                    .unwrap();
            }
        }
        log::trace!("End frame");
        PrepareResult::DrawRecord
    }

    fn draw(
        &mut self,
        layout: &B::PipelineLayout,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        resources: &Resources,
    ) {
        let (
            active_camera,
            cameras,
            sprite_sheet_storage,
            tex_storage,
            visibilities,
            hiddens,
            hidden_props,
            sprite_renders,
            global_transforms,
            texture_handles,
        ) = <(
            Option<Read<'_, ActiveCamera>>,
            ReadStorage<'_, Camera>,
            Read<'_, AssetStorage<SpriteSheet<B>>>,
            Read<'_, AssetStorage<Texture<B>>>,
            Option<Read<'_, SpriteVisibility>>,
            ReadStorage<'_, Hidden>,
            ReadStorage<'_, HiddenPropagate>,
            ReadStorage<'_, SpriteRender<B>>,
            ReadStorage<'_, GlobalTransform>,
            ReadStorage<'_, Handle<Texture<B>>>,
        ) as SystemData>::fetch(resources);

        encoder.bind_graphics_descriptor_sets(
            layout,
            0, // 1 for tex
            std::iter::once(self.projview_set.as_ref().unwrap().raw()),
            std::iter::empty::<u32>(),
        );

        for (n, (tex_id, (batch_data, batch_constant_id))) in self.batches.iter().enumerate() {
            log::trace!("drawing batch: {},{},{}", n, tex_id, batch_constant_id);
            encoder.push_constants(layout, ShaderStageFlags::FRAGMENT, 0, &[*batch_constant_id]);

            if let Some(mut buffer) = self.vertex_buffers.get(*batch_constant_id as usize) {
                encoder.bind_vertex_buffers(
                    1,
                    Some((
                        buffer.as_ref().unwrap().raw(),
                        batch_data.len() as u64 * std::mem::size_of::<SpriteArgs>() as u64,
                    )),
                );
            } else {
                log::warn!("Failed to get vertex buffer");
            }
            // Draw and wrap
            encoder.draw(0..6, 0..batch_data.len() as u32);
        }

        self.batches.clear();
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &Resources) {
        unimplemented!()
    }
}
