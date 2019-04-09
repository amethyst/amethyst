use crate::{
    camera::{ActiveCamera, Camera},
    mtl::{Material, MaterialDefaults},
    skinning::JointTransforms,
    types::{Mesh, Texture},
    visibility::Visibility,
    sprite_visibility::SpriteVisibility,
    sprite::{SpriteSheet, Sprite, SpriteRender, SpriteSheetFormat, SpriteSheetHandle },
    hidden::{Hidden, HiddenPropagate},
    pass::util,
    pod::{self, ViewArgs, SpriteArgs, IntoPod},
    batch::{BatchData, BatchPrimitives}
};
use glsl_layout::AsStd140;

use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    math::{Vector4, Vector2},
    ecs::{Join, Read, ReadExpect, ReadStorage, Resources, SystemData},
    transform::GlobalTransform,
};
use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{PrepareResult, SimpleGraphicsPipeline, SimpleGraphicsPipelineDesc, Layout, SetLayout},
        GraphContext, NodeBuffer, NodeImage,
    },
    hal::{
        device::Device,
        adapter::PhysicalDevice,
        buffer::Usage as BufferUsage,
        pso::{
            BlendState, ColorBlendDesc, ColorMask, DepthStencilDesc, EntryPoint, GraphicsShaderSet,
            Specialization, Element, ElemStride, InstanceRate,
            DescriptorSetLayoutBinding, DescriptorType, ShaderStageFlags, Descriptor, DescriptorSetWrite,
        },
        format::Format,
        Backend,
    },
    mesh::{AsVertex, PosTex},
    resource::{DescriptorSet, DescriptorSetLayout, Handle as RendyHandle, Escape},
    shader::Shader,
};
use fnv::FnvHashMap;
use smallvec::{SmallVec, smallvec};
use derivative::Derivative;


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

        log::trace!("Loading shader module '{:#?}'", *super::BASIC_VERTEX);
        storage.push(unsafe { super::SPRITE_VERTEX.module(factory).unwrap() });

        log::trace!("Loading shader module '{:#?}'", *super::FLAT_FRAGMENT);
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
        if let Some((color, _)) = self.transparency {
            vec![color]
        } else {
            vec![ColorBlendDesc(ColorMask::ALL, BlendState::ALPHA)]
        }
    }

    fn depth_stencil(&self) -> Option<DepthStencilDesc> {
        if let Some((_, stencil)) = self.transparency {
            stencil
        } else {
            None
        }
    }


    fn vertices(
        &self,
    ) -> Vec<(
        Vec<Element<Format>>,
        ElemStride,
        InstanceRate,
    )> {
        vec![PosTex::VERTEX.gfx_vertex_input_desc(0)]
    }

    fn layout(&self) -> Layout {
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
                }
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

        let mut projview_buffer: Option<Escape<rendy::resource::Buffer<B>>> = None;
        let mut tex_buffer: Option<Escape<rendy::resource::Buffer<B>>> = None;
        let mut tex_id_buffer: Option<Escape<rendy::resource::Buffer<B>>> = None;

        let limits = factory.physical().limits();

        let projview_size = util::align_size::<pod::ViewArgs>(limits.min_uniform_buffer_offset_alignment, 1);
        let projview_set: Option<Escape<DescriptorSet<B>>>;

        util::ensure_buffer(
            &factory,
            &mut projview_buffer,
            BufferUsage::UNIFORM,
            rendy::memory::Dynamic,
            projview_size,
        ).unwrap();

        util::ensure_buffer(
            &factory,
            &mut tex_id_buffer,
            BufferUsage::UNIFORM,
            rendy::memory::Dynamic,
            4,
        ).unwrap();

        let projview_set = factory
            .create_descriptor_set(set_layouts[0].clone())
            .unwrap();

        let tex_set = factory
            .create_descriptor_set(set_layouts[1].clone())
            .unwrap();

        Ok(DrawFlat2D {
            projview_buffer,
            tex_id_buffer,
            vertex_buffers: Vec::new(),
            batches: FnvHashMap::default(),
            projview_set: Some(projview_set),
            tex_set: Some(tex_set),
            ubo_offset_align: limits.min_uniform_buffer_offset_alignment,
        })
    }
}

#[derive(Debug)]
pub struct DrawFlat2D<B: Backend> {
    projview_buffer: Option<Escape<rendy::resource::Buffer<B>>>,
    projview_set:  Option<Escape<DescriptorSet<B>>>,
    vertex_buffers: Vec<Escape<rendy::resource::Buffer<B>>>,
    tex_id_buffer: Option<Escape<rendy::resource::Buffer<B>>>,
    tex_set:  Option<Escape<DescriptorSet<B>>>,
    batches: FnvHashMap<u32, (Vec<pod::SpriteArgs>, u32)>,
    ubo_offset_align: u64,
}
impl<B: Backend> DrawFlat2D<B> {

    #[inline]
    fn desc_write<'a>(
        set: &'a B::DescriptorSet,
        binding: u32,
        descriptor: Descriptor<'a, B>,
    ) -> DescriptorSetWrite<'a, B, Option<Descriptor<'a, B>>> {
        DescriptorSetWrite {
            set,
            binding,
            array_offset: 0,
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

        let (camera_position, projview) = util::prepare_camera(&active_camera, &cameras, &global_transforms);

        match visibilities {
            None => {
                for (sprite_render, global, _, _) in (
                    &sprite_renders,
                    &global_transforms,
                    !&hiddens,
                    !&hidden_props,
                ).join()
                    {
                        let tex_id = sprite_sheet_storage.get(&sprite_render.sprite_sheet).unwrap().texture.id();

                        let batch_data = SpriteArgs {
                            dir_x: Vector2::new(1.0, 1.0).into_pod(),
                            dir_y: Vector2::new(1.0, 1.0).into_pod(),
                            pos: Vector2::new(1.0, 1.0).into_pod(),
                            depth: 1.0.into()
                        };

                        if let Some(batch) = self.batches.get_mut(&tex_id) {
                            batch.0.push(batch_data);
                        } else {
                            let mut newbatch = Vec::with_capacity(1024);
                            newbatch.push(batch_data);
                            self.batches.insert(tex_id, (newbatch, tex_id) );
                        }


                        //BatchSprite::insert_batch(self.batches.entry(tex), 0, &[batch_data]);

                       // self.batch.add_sprite(
                      //      &sprite_render,
                      //      Some(&global),
                      //      &sprite_sheet_storage,
                      //      &tex_storage,
                       // );
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

        let desc_writes: SmallVec<[DescriptorSetWrite<'_, B, Option<Descriptor<'_, B>>>; 32]>;
        // build our texture descriptor once
        // prepare each texture here, we should check if theyve changed..
        for (n, batch) in self.batches.iter().enumerate() {
            if let Some(texture) = tex_storage.get_by_id(*batch.0) {
                let descriptor = Descriptor::CombinedImageSampler(
                    texture.view().raw(),
                    rendy::hal::image::Layout::ShaderReadOnlyOptimal,
                    texture.sampler().raw(),
                );
                desc_writes.push(
                    DescriptorSetWrite {
                        set: self.tex_set,
                        binding: 1,
                        array_offset: n,
                        descriptors: Some(descriptor),
                    }
                );
            } else {
                log::warn!("Failed to fetch texture");
            }
        }

        factory.write_descriptor_sets(desc_writes);

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

        encoder.push_constants(
            layout,
            ShaderStageFlags::FRAGMENT,
            0,
            &[0]
        );



    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &Resources) {
        unimplemented!()
    }
}
