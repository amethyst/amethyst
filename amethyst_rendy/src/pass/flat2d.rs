use amethyst_assets::AssetStorage;
use amethyst_core::{
    ecs::{systems::ResourceSet, *},
    transform::Transform,
};
use derivative::Derivative;
use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{PrepareResult, RenderGroup, RenderGroupDesc},
        GraphContext, NodeBuffer, NodeImage,
    },
    hal::{self, device::Device, pso},
    mesh::AsVertex,
    shader::Shader,
};
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{
    batch::{GroupIterator, OneLevelBatch, OrderedOneLevelBatch},
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    pod::SpriteArgs,
    resources::Tint,
    sprite::{SpriteRender, SpriteSheet, Sprites},
    sprite_visibility::SpriteVisibility,
    submodules::{DynamicVertexBuffer, FlatEnvironmentSub, TextureId, TextureSub},
    system::GraphAuxData,
    types::{Backend, Texture},
    util,
};

/// Draw opaque sprites without lighting.
#[derive(Clone, Debug, PartialEq, Derivative)]
#[derivative(Default(bound = ""))]
pub struct DrawFlat2DDesc;

impl DrawFlat2DDesc {
    /// Create instance of `DrawFlat2D` render group
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> RenderGroupDesc<B, GraphAuxData> for DrawFlat2DDesc {
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &GraphAuxData,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, GraphAuxData>>, pso::CreationError> {
        #[cfg(feature = "profiler")]
        profile_scope!("build");

        let env = FlatEnvironmentSub::new(factory)?;
        let textures = TextureSub::new(factory)?;
        let vertex = DynamicVertexBuffer::new();

        let (pipeline, pipeline_layout) = build_sprite_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            false,
            vec![env.raw_layout(), textures.raw_layout()],
        )?;

        Ok(Box::new(DrawFlat2D::<B> {
            pipeline,
            pipeline_layout,
            env,
            textures,
            vertex,
            sprites: Default::default(),
        }))
    }
}

/// Draws opaque 2D sprites to the screen without lighting.
#[derive(Debug)]
pub struct DrawFlat2D<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: FlatEnvironmentSub<B>,
    textures: TextureSub<B>,
    vertex: DynamicVertexBuffer<B, SpriteArgs>,
    sprites: OneLevelBatch<TextureId, SpriteArgs>,
}

impl<B: Backend> RenderGroup<B, GraphAuxData> for DrawFlat2D<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        aux: &GraphAuxData,
    ) -> PrepareResult {
        #[cfg(feature = "profiler")]
        profile_scope!("prepare opaque");

        let GraphAuxData { world, resources } = aux;

        let (sprite_sheet_storage, sprites_storage, tex_storage, visibility) =
            <(
                Read<AssetStorage<SpriteSheet>>,
                Read<AssetStorage<Sprites>>,
                Read<AssetStorage<Texture>>,
                Read<SpriteVisibility>,
            )>::fetch(resources);

        self.env.process(factory, index, world, resources);

        let sprites_ref = &mut self.sprites;
        let textures_ref = &mut self.textures;

        sprites_ref.clear_inner();

        {
            #[cfg(feature = "profiler")]
            profile_scope!("gather_visibility");

            let mut query = <(&SpriteRender, &Transform, Option<&Tint>)>::query();

            visibility
                .visible_unordered
                .iter()
                .filter_map(|entity| Some((entity, query.get(*world, *entity).ok()?)))
                .filter_map(|(entity, (sprite_render, global, tint))| {
                    let sprite_sheet = sprite_sheet_storage.get(&sprite_render.sprite_sheet)?;
                    let sprites = sprites_storage.get(&sprite_sheet.sprites)?.build_sprites();

                    let sprite = &sprites[sprite_render.sprite_number];

                    let batch_data = SpriteArgs::from_data(&sprite, &global, tint);

                    let (tex_id, _) = textures_ref.insert(
                        factory,
                        resources,
                        &sprite_sheet.texture,
                        hal::image::Layout::ShaderReadOnlyOptimal,
                    )?;
                    Some((tex_id, batch_data))
                })
                .for_each_group(|tex_id, batch_data| {
                    sprites_ref.insert(tex_id, batch_data.drain(..))
                });
        }

        self.textures.maintain(factory, resources);

        {
            #[cfg(feature = "profiler")]
            profile_scope!("write");

            sprites_ref.prune();
            self.vertex.write(
                factory,
                index,
                self.sprites.count() as u64,
                self.sprites.data(),
            );
        }

        PrepareResult::DrawRecord
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _world: &GraphAuxData,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("draw opaque");

        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, layout, 0, &mut encoder);
        self.vertex.bind(index, 0, 0, &mut encoder);
        for (&tex, range) in self.sprites.iter() {
            if self.textures.loaded(tex) {
                self.textures.bind(layout, 1, tex, &mut encoder);
                unsafe {
                    encoder.draw(0..4, range);
                }
            }
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _world: &GraphAuxData) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}
/// Describes drawing transparent sprites without lighting.
#[derive(Clone, Debug, PartialEq, Derivative)]
#[derivative(Default(bound = ""))]
pub struct DrawFlat2DTransparentDesc;

impl DrawFlat2DTransparentDesc {
    /// Create instance of `DrawFlat2D` render group
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> RenderGroupDesc<B, GraphAuxData> for DrawFlat2DTransparentDesc {
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &GraphAuxData,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, GraphAuxData>>, pso::CreationError> {
        #[cfg(feature = "profiler")]
        profile_scope!("build_trans");

        let env = FlatEnvironmentSub::new(factory)?;
        let textures = TextureSub::new(factory)?;
        let vertex = DynamicVertexBuffer::new();

        let (pipeline, pipeline_layout) = build_sprite_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            true,
            vec![env.raw_layout(), textures.raw_layout()],
        )?;

        Ok(Box::new(DrawFlat2DTransparent::<B> {
            pipeline,
            pipeline_layout,
            env,
            textures,
            vertex,
            sprites: Default::default(),
            change: Default::default(),
        }))
    }
}

/// Draws transparent sprites without lighting.
#[derive(Debug)]
pub struct DrawFlat2DTransparent<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: FlatEnvironmentSub<B>,
    textures: TextureSub<B>,
    vertex: DynamicVertexBuffer<B, SpriteArgs>,
    sprites: OrderedOneLevelBatch<TextureId, SpriteArgs>,
    change: util::ChangeDetection,
}

impl<B: Backend> RenderGroup<B, GraphAuxData> for DrawFlat2DTransparent<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        aux: &GraphAuxData,
    ) -> PrepareResult {
        #[cfg(feature = "profiler")]
        profile_scope!("prepare transparent");

        let GraphAuxData { world, resources } = aux;

        let (sprite_sheet_storage, sprites_storage, tex_storage, visibility) =
            <(
                Read<AssetStorage<SpriteSheet>>,
                Read<AssetStorage<Sprites>>,
                Read<AssetStorage<Texture>>,
                Read<SpriteVisibility>,
            )>::fetch(resources);

        self.env.process(factory, index, world, resources);

        let sprites_ref = &mut self.sprites;
        let textures_ref = &mut self.textures;

        sprites_ref.swap_clear();
        let mut changed = false;

        {
            #[cfg(feature = "profiler")]
            profile_scope!("gather_visibility");

            let mut query = <(&SpriteRender, &Transform, Option<&Tint>)>::query();

            visibility
                .visible_ordered
                .iter()
                .filter_map(|entity| Some((entity, query.get(*world, *entity).ok()?)))
                .filter_map(|(entity, (sprite_render, global, tint))| {
                    let sprite_sheet = sprite_sheet_storage.get(&sprite_render.sprite_sheet)?;
                    let sprites = sprites_storage.get(&sprite_sheet.sprites)?.build_sprites();

                    let sprite = &sprites[sprite_render.sprite_number];

                    let batch_data = SpriteArgs::from_data(&sprite, &global, tint);

                    let (tex_id, this_changed) = textures_ref.insert(
                        factory,
                        resources,
                        &sprite_sheet.texture,
                        hal::image::Layout::ShaderReadOnlyOptimal,
                    )?;

                    changed = changed || this_changed;
                    Some((tex_id, batch_data))
                })
                .for_each_group(|tex_id, batch_data| {
                    sprites_ref.insert(tex_id, batch_data.drain(..))
                });
        }

        self.textures.maintain(factory, resources);
        changed = changed || sprites_ref.changed();

        {
            #[cfg(feature = "profiler")]
            profile_scope!("write");

            self.vertex.write(
                factory,
                index,
                self.sprites.count() as u64,
                Some(self.sprites.data()),
            );
        }

        self.change.prepare_result(index, changed)
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _world: &GraphAuxData,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("draw transparent");

        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, layout, 0, &mut encoder);
        self.vertex.bind(index, 0, 0, &mut encoder);
        for (&tex, range) in self.sprites.iter() {
            if self.textures.loaded(tex) {
                self.textures.bind(layout, 1, tex, &mut encoder);
                unsafe {
                    encoder.draw(0..4, range);
                }
            }
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &GraphAuxData) {
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
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), pso::CreationError> {
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
                .with_vertex_desc(&[(SpriteArgs::vertex(), pso::VertexInputRate::Instance(1))])
                .with_input_assembler(pso::InputAssemblerDesc::new(pso::Primitive::TriangleStrip))
                .with_shaders(util::simple_shader_set(
                    &shader_vertex,
                    Some(&shader_fragment),
                ))
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                .with_blend_targets(vec![pso::ColorBlendDesc {
                    mask: pso::ColorMask::ALL,
                    blend: if transparent {
                        Some(pso::BlendState::PREMULTIPLIED_ALPHA)
                    } else {
                        None
                    },
                }])
                .with_depth_test(pso::DepthTest {
                    fun: pso::Comparison::Greater,
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
