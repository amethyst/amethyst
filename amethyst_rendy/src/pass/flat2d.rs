use crate::{
    batch::{GroupIterator, OneLevelBatch, OrderedOneLevelBatch},
    hidden::{Hidden, HiddenPropagate},
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    pod::SpriteArgs,
    sprite::{SpriteRender, SpriteSheet},
    sprite_visibility::SpriteVisibility,
    submodules::{DynamicVertex, FlatEnvironmentSub, TextureId, TextureSub},
    types::Texture,
    util,
};
use amethyst_assets::AssetStorage;
use amethyst_core::{
    ecs::{Join, Read, ReadExpect, ReadStorage, Resources, SystemData},
    transform::GlobalTransform,
};
use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{PrepareResult, RenderGroup, RenderGroupDesc},
        BufferAccess, GraphContext, ImageAccess, NodeBuffer, NodeImage,
    },
    hal::{self, device::Device, pso, Backend},
    mesh::AsVertex,
    shader::Shader,
};

/// Draw opaque sprites without lighting.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawFlat2DDesc;

impl DrawFlat2DDesc {
    /// Create instance of `DrawFlat2D` render group
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> RenderGroupDesc<B, Resources> for DrawFlat2DDesc {
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &Resources,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, Resources>>, failure::Error> {
        let env = FlatEnvironmentSub::new(factory)?;
        let textures = TextureSub::new(factory)?;
        let vertex = DynamicVertex::new();

        let (pipeline, pipeline_layout) = build_sprite_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            false,
            vec![env.raw_layout(), textures.raw_layout()],
        )?;

        Ok(Box::new(DrawFlat2D::<B> {
            pipeline: pipeline,
            pipeline_layout,
            env,
            textures,
            vertex,
            sprites: Default::default(),
        }))
    }
}

#[derive(Debug)]
pub struct DrawFlat2D<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: FlatEnvironmentSub<B>,
    textures: TextureSub<B>,
    vertex: DynamicVertex<B, SpriteArgs>,
    sprites: OneLevelBatch<TextureId, SpriteArgs>,
}

impl<B: Backend> RenderGroup<B, Resources> for DrawFlat2D<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        resources: &Resources,
    ) -> PrepareResult {
        let (
            sprite_sheet_storage,
            tex_storage,
            visibilities,
            hiddens,
            hidden_props,
            sprite_renders,
            global_transforms,
        ) = <(
            Read<'_, AssetStorage<SpriteSheet<B>>>,
            Read<'_, AssetStorage<Texture<B>>>,
            Option<Read<'_, SpriteVisibility>>,
            ReadStorage<'_, Hidden>,
            ReadStorage<'_, HiddenPropagate>,
            ReadStorage<'_, SpriteRender<B>>,
            ReadStorage<'_, GlobalTransform>,
        )>::fetch(resources);

        self.env.process(factory, index, resources);
        self.textures.maintain();

        let sprites_ref = &mut self.sprites;
        let textures_ref = &mut self.textures;

        sprites_ref.clear_inner();

        match visibilities {
            None => {
                (
                    &sprite_renders,
                    &global_transforms,
                    !&hiddens,
                    !&hidden_props,
                )
                    .join()
                    .filter_map(|(sprite_render, global, _, _)| {
                        let (batch_data, texture) = SpriteArgs::from_data(
                            &tex_storage,
                            &sprite_sheet_storage,
                            &sprite_render,
                            &global,
                        )?;
                        let (tex_id, _) = textures_ref.insert(factory, resources, texture)?;
                        Some((tex_id, batch_data))
                    })
                    .for_each_group(|tex_id, batch_data| {
                        sprites_ref.insert(tex_id, batch_data.drain(..))
                    });
            }
            Some(ref visibility) => {
                (
                    &sprite_renders,
                    &global_transforms,
                    &visibility.visible_unordered,
                )
                    .join()
                    .filter_map(|(sprite_render, global, _)| {
                        let (batch_data, texture) = SpriteArgs::from_data(
                            &tex_storage,
                            &sprite_sheet_storage,
                            &sprite_render,
                            &global,
                        )?;
                        let (tex_id, _) = textures_ref.insert(factory, resources, texture)?;
                        Some((tex_id, batch_data))
                    })
                    .for_each_group(|tex_id, batch_data| {
                        sprites_ref.insert(tex_id, batch_data.drain(..))
                    });
            }
        }

        sprites_ref.prune();
        self.vertex.write(
            factory,
            index,
            self.sprites.count() as u64,
            self.sprites.data(),
        );

        PrepareResult::DrawRecord
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _resources: &Resources,
    ) {
        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, layout, 0, &mut encoder);
        self.vertex.bind(index, 0, &mut encoder);
        for (&tex, range) in self.sprites.iter() {
            self.textures.bind(layout, 1, tex, &mut encoder);
            encoder.draw(0..6, range);
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &Resources) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}
/// Draw transparent sprites without lighting.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawFlat2DTransparentDesc;

impl DrawFlat2DTransparentDesc {
    /// Create instance of `DrawFlat2D` render group
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> RenderGroupDesc<B, Resources> for DrawFlat2DTransparentDesc {
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &Resources,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, Resources>>, failure::Error> {
        let env = FlatEnvironmentSub::new(factory)?;
        let textures = TextureSub::new(factory)?;
        let vertex = DynamicVertex::new();

        let (pipeline, pipeline_layout) = build_sprite_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            true,
            vec![env.raw_layout(), textures.raw_layout()],
        )?;

        Ok(Box::new(DrawFlat2DTransparent::<B> {
            pipeline: pipeline,
            pipeline_layout,
            env,
            textures,
            vertex,
            sprites: Default::default(),
            change: Default::default(),
        }))
    }
}

#[derive(Debug)]
pub struct DrawFlat2DTransparent<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: FlatEnvironmentSub<B>,
    textures: TextureSub<B>,
    vertex: DynamicVertex<B, SpriteArgs>,
    sprites: OrderedOneLevelBatch<TextureId, SpriteArgs>,
    change: util::ChangeDetection,
}

impl<B: Backend> RenderGroup<B, Resources> for DrawFlat2DTransparent<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        resources: &Resources,
    ) -> PrepareResult {
        let (sprite_sheet_storage, tex_storage, visibility, sprite_renders, global_transforms) =
            <(
                Read<'_, AssetStorage<SpriteSheet<B>>>,
                Read<'_, AssetStorage<Texture<B>>>,
                ReadExpect<'_, SpriteVisibility>,
                ReadStorage<'_, SpriteRender<B>>,
                ReadStorage<'_, GlobalTransform>,
            )>::fetch(resources);

        self.env.process(factory, index, resources);
        self.textures.maintain();
        self.sprites.swap_clear();
        let mut changed = false;

        let sprites_ref = &mut self.sprites;
        let textures_ref = &mut self.textures;

        let mut joined = (&sprite_renders, &global_transforms).join();
        visibility
            .visible_ordered
            .iter()
            .filter_map(|e| joined.get_unchecked(e.id()))
            .filter_map(|(sprite_render, global)| {
                let (batch_data, texture) = SpriteArgs::from_data(
                    &tex_storage,
                    &sprite_sheet_storage,
                    &sprite_render,
                    &global,
                )?;
                let (tex_id, this_changed) = textures_ref.insert(factory, resources, texture)?;
                changed = changed || this_changed;
                Some((tex_id, batch_data))
            })
            .for_each_group(|tex_id, batch_data| {
                sprites_ref.insert(tex_id, batch_data.drain(..));
            });

        changed = changed || self.sprites.changed();
        self.vertex.write(
            factory,
            index,
            self.sprites.count() as u64,
            Some(self.sprites.data()),
        );

        self.change.prepare_result(index, changed)
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _resources: &Resources,
    ) {
        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, layout, 0, &mut encoder);
        self.vertex.bind(index, 0, &mut encoder);
        for (&tex, range) in self.sprites.iter() {
            self.textures.bind(layout, 1, tex, &mut encoder);
            encoder.draw(0..6, range);
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &Resources) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}

fn build_sprite_pipeline<B: Backend>(
    factory: &Factory<B>,
    subpass: hal::pass::Subpass<'_, B>,
    framebuffer_width: u32,
    framebuffer_height: u32,
    transparent: bool,
    layouts: Vec<&B::DescriptorSetLayout>,
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), failure::Error> {
    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
    }?;

    let shader_vertex = unsafe { super::SPRITE_VERTEX.module(factory).unwrap() };
    let shader_fragment = unsafe { super::SPRITE_FRAGMENT.module(factory).unwrap() };

    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                .with_vertex_desc(&[(SpriteArgs::VERTEX, 1)])
                .with_shaders(util::simple_shader_set(
                    &shader_vertex,
                    Some(&shader_fragment),
                ))
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                .with_blend_targets(vec![pso::ColorBlendDesc(
                    pso::ColorMask::ALL,
                    if transparent {
                        pso::BlendState::ALPHA
                    } else {
                        pso::BlendState::Off
                    },
                )])
                .with_depth_test(pso::DepthTest::On {
                    fun: pso::Comparison::Less,
                    write: !transparent,
                }),
        )
        .build(factory, None);

    unsafe {
        factory.destroy_shader_module(shader_vertex);
        factory.destroy_shader_module(shader_fragment);
    }

    match pipes {
        Err(e) => {
            unsafe {
                factory.device().destroy_pipeline_layout(pipeline_layout);
            }
            Err(e)
        }
        Ok(mut pipes) => Ok((pipes.remove(0), pipeline_layout)),
    }
}
