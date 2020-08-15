use crate::{
    batch::{GroupIterator, OneLevelBatch, OrderedOneLevelBatch},
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    resources::Tint,
    sprite::SpriteSheet,
    sprite_visibility::SpriteVisibility,
    submodules::{DynamicVertexBuffer, FlatEnvironmentSub, TextureId, TextureSub},
    types::{Backend, Texture},
    util,
};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    ecs::{Component, Join, Read, ReadExpect, ReadStorage, SystemData, World},
    transform::Transform,
    Hidden, HiddenPropagate,
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
    shader::{Shader, SpirvShader},
};

use static_assertions::_core::marker::PhantomData;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// Define drawing opaque 3d meshes with specified shaders and texture set
pub trait Base2DPassDef: 'static + std::fmt::Debug + Send + Sync {
    /// The human readable name of this pass
    const NAME: &'static str;

    type SpriteComponent: Component;
    type SpriteData: AsVertex;

    /// Returns the vertex `SpirvShader` which will be used for this pass
    fn vertex_shader() -> &'static SpirvShader;

    /// Returns the fragment `SpirvShader` which will be used for this pass
    fn fragment_shader() -> &'static SpirvShader;

    fn get_args<'a>(
        tex_storage: &AssetStorage<Texture>,
        sprite_storage: &'a AssetStorage<SpriteSheet>,
        sprite_render: &Self::SpriteComponent,
        transform: &Transform,
        tint: Option<&Tint>,
    ) -> Option<(Self::SpriteData, &'a [Handle<Texture>])>;
}

/// Draw opaque 3d meshes with specified shaders and texture set
#[derive(Clone, Derivative)]
#[derivative(Debug(bound = ""), Default(bound = ""))]
pub struct DrawBase2DDesc<B: Backend, T: Base2DPassDef> {
    marker: PhantomData<(B, T)>,
}

impl<B: Backend, T: Base2DPassDef> DrawBase2DDesc<B, T> {
    /// Create pass in default configuration
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend, T: Base2DPassDef> RenderGroupDesc<B, World> for DrawBase2DDesc<B, T> {
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &World,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, World>>, failure::Error> {
        #[cfg(feature = "profiler")]
        profile_scope!("build");

        let env = FlatEnvironmentSub::new(factory)?;
        let textures = TextureSub::new(factory)?;
        let vertex = DynamicVertexBuffer::new();

        let (pipeline, pipeline_layout) = build_sprite_pipeline::<B, T>(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            false,
            vec![env.raw_layout(), textures.raw_layout()],
        )?;

        Ok(Box::new(DrawBase2D::<B, T> {
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
pub struct DrawBase2D<B: Backend, T: Base2DPassDef> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: FlatEnvironmentSub<B>,
    textures: TextureSub<B>,
    vertex: DynamicVertexBuffer<B, T::SpriteData>,
    sprites: OneLevelBatch<Vec<TextureId>, T::SpriteData>,
}

impl<B: Backend, T: Base2DPassDef> RenderGroup<B, World> for DrawBase2D<B, T> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        world: &World,
    ) -> PrepareResult {
        #[cfg(feature = "profiler")]
        profile_scope!("prepare opaque");

        let (
            sprite_sheet_storage,
            tex_storage,
            visibility,
            hiddens,
            hidden_props,
            sprite_renders,
            transforms,
            tints,
        ) = <(
            Read<'_, AssetStorage<SpriteSheet>>,
            Read<'_, AssetStorage<Texture>>,
            ReadExpect<'_, SpriteVisibility>,
            ReadStorage<'_, Hidden>,
            ReadStorage<'_, HiddenPropagate>,
            ReadStorage<'_, T::SpriteComponent>,
            ReadStorage<'_, Transform>,
            ReadStorage<'_, Tint>,
        )>::fetch(world);

        self.env.process(factory, index, world);

        let sprites_ref = &mut self.sprites;
        let textures_ref = &mut self.textures;

        sprites_ref.clear_inner();

        {
            #[cfg(feature = "profiler")]
            profile_scope!("gather_visibility");

            (
                &sprite_renders,
                &transforms,
                tints.maybe(),
                &visibility.visible_unordered,
            )
                .join()
                .filter_map(|(sprite_render, global, tint, _)| {
                    let (batch_data, textures) = T::get_args(
                        &tex_storage,
                        &sprite_sheet_storage,
                        &sprite_render,
                        &global,
                        tint,
                    )?;

                    let tex_ids: Vec<TextureId> = textures
                        .iter()
                        .map(|texture| {
                            let (tex_id, _) = textures_ref
                                .insert(
                                    factory,
                                    world,
                                    texture,
                                    hal::image::Layout::ShaderReadOnlyOptimal,
                                )
                                .unwrap();
                            tex_id
                        })
                        .collect();

                    Some((tex_ids, batch_data))
                })
                .for_each_group(|tex_ids, batch_data| {
                    sprites_ref.insert(tex_ids, batch_data.drain(..))
                });
        }

        self.textures.maintain(factory, world);

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
        _world: &World,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("draw opaque");

        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, layout, 0, &mut encoder);
        self.vertex.bind(index, 0, 0, &mut encoder);
        for (texs, range) in self.sprites.iter() {
            for (num, &tex) in texs.iter().enumerate() {
                if self.textures.loaded(tex) {
                    self.textures.bind(layout, 1, tex, &mut encoder);
                    unsafe {
                        encoder.draw(0..4, range.clone());
                    }
                }
            }
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _world: &World) {
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
pub struct DrawBase2DTransparentDesc<B: Backend, T: Base2DPassDef> {
    marker: PhantomData<(B, T)>,
}

impl<B: Backend, T: Base2DPassDef> DrawBase2DTransparentDesc<B, T> {
    /// Create instance of `DrawFlat2D` render group
    pub fn new() -> Self {
        Default::default()
    }
}
impl<B: Backend, T: Base2DPassDef> RenderGroupDesc<B, World> for DrawBase2DTransparentDesc<B, T> {
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _world: &World,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, World>>, failure::Error> {
        #[cfg(feature = "profiler")]
        profile_scope!("build_trans");

        let env = FlatEnvironmentSub::new(factory)?;
        let textures = TextureSub::new(factory)?;
        let vertex = DynamicVertexBuffer::new();

        let (pipeline, pipeline_layout) = build_sprite_pipeline::<B, T>(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            true,
            vec![env.raw_layout(), textures.raw_layout()],
        )?;

        Ok(Box::new(DrawBase2DTransparent::<B, T> {
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
pub struct DrawBase2DTransparent<B: Backend, T: Base2DPassDef> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: FlatEnvironmentSub<B>,
    textures: TextureSub<B>,
    vertex: DynamicVertexBuffer<B, T::SpriteData>,
    sprites: OrderedOneLevelBatch<Vec<TextureId>, T::SpriteData>,
    change: util::ChangeDetection,
}

impl<B: Backend, T: Base2DPassDef> RenderGroup<B, World> for DrawBase2DTransparent<B, T> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        world: &World,
    ) -> PrepareResult {
        #[cfg(feature = "profiler")]
        profile_scope!("prepare transparent");

        let (sprite_sheet_storage, tex_storage, visibility, sprite_renders, transforms, tints) =
            <(
                Read<'_, AssetStorage<SpriteSheet>>,
                Read<'_, AssetStorage<Texture>>,
                ReadExpect<'_, SpriteVisibility>,
                ReadStorage<'_, T::SpriteComponent>,
                ReadStorage<'_, Transform>,
                ReadStorage<'_, Tint>,
            )>::fetch(world);

        self.env.process(factory, index, world);
        self.sprites.swap_clear();
        let mut changed = false;

        let sprites_ref = &mut self.sprites;
        let textures_ref = &mut self.textures;

        {
            #[cfg(feature = "profiler")]
            profile_scope!("gather_sprites_trans");

            let mut joined = (&sprite_renders, &transforms, tints.maybe()).join();
            visibility
                .visible_ordered
                .iter()
                .filter_map(|e| joined.get_unchecked(e.id()))
                .filter_map(|(sprite_render, global, tint)| {
                    let (batch_data, textures) = T::get_args(
                        &tex_storage,
                        &sprite_sheet_storage,
                        &sprite_render,
                        &global,
                        tint,
                    )?;

                    let tex_ids: Vec<TextureId> = textures
                        .iter()
                        .map(|texture| {
                            let (tex_id, this_changed) = textures_ref
                                .insert(
                                    factory,
                                    world,
                                    texture,
                                    hal::image::Layout::ShaderReadOnlyOptimal,
                                )
                                .unwrap();
                            changed = changed || this_changed;
                            tex_id
                        })
                        .collect();

                    Some((tex_ids, batch_data))
                })
                .for_each_group(|tex_ids, batch_data| {
                    sprites_ref.insert(tex_ids, batch_data.drain(..));
                });
        }
        self.textures.maintain(factory, world);
        changed = changed || self.sprites.changed();

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
        _world: &World,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("draw transparent");

        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, layout, 0, &mut encoder);
        self.vertex.bind(index, 0, 0, &mut encoder);
        for (texs, range) in self.sprites.iter() {
            for (num, &tex) in texs.iter().enumerate() {
                if self.textures.loaded(tex) {
                    self.textures.bind(layout, 1, tex, &mut encoder);
                    unsafe {
                        encoder.draw(0..4, range.clone());
                    }
                }
            }
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &World) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}

fn build_sprite_pipeline<B: Backend, T: Base2DPassDef>(
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
                .with_vertex_desc(&[(T::SpriteData::vertex(), pso::VertexInputRate::Instance(1))])
                .with_input_assembler(pso::InputAssemblerDesc::new(hal::Primitive::TriangleStrip))
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