use amethyst::{
	assets::{AssetStorage, Handle, Loader},
	core::{
        ecs::{Join, Read, ReadExpect, ReadStorage, SystemData, World, DispatcherBuilder},
        transform::Transform,
        Hidden, HiddenPropagate,
	},
	renderer::{
        sprite_visibility::SpriteVisibilitySortingSystem,
		batch::{OrderedOneLevelBatch,OneLevelBatch,GroupIterator},
		pipeline::{PipelineDescBuilder, PipelinesBuilder},
		rendy::{
			command::{QueueId, RenderPassEncoder},
			factory::Factory,
			graph::{
				render::{PrepareResult, RenderGroup, RenderGroupDesc},
				GraphContext,
				NodeBuffer,
				NodeImage,
			},
			hal::{
				self,
				device::Device,
				format::Format,
				image::{self, Anisotropic, Filter, PackedColor, SamplerInfo, WrapMode},
				pso,
			},

			mesh::{AsAttribute, AsVertex, Color, TexCoord, VertexFormat},
			shader::Shader,
			texture::TextureBuilder,
            hal::pso::ShaderStageFlags, shader::SpirvShader
		},
		submodules::{DynamicIndexBuffer, DynamicVertexBuffer, TextureId, TextureSub,FlatEnvironmentSub},
		types::{Backend, TextureData},
		util,
		Texture,
        bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
        pod::SpriteArgs,
        resources::Tint,
        sprite::{SpriteRender, SpriteSheet},
        sprite_visibility::SpriteVisibility,
    },
	shrev::{EventChannel, ReaderId},
	window::Window,
	winit::Event,
};
use derivative::Derivative;
use std::{
	borrow::Cow,
	path::PathBuf,
	sync::{Arc, Mutex},
};
use amethyst_error::Error;

lazy_static::lazy_static! {
    static ref SPRITE_VERTEX: SpirvShader = SpirvShader::new(
        include_bytes!("../assets/shaders/compiled/vertex/custom.vert.spv").to_vec(),
        ShaderStageFlags::VERTEX,
        "main",
    );

    static ref SPRITE_FRAGMENT: SpirvShader = SpirvShader::new(
        include_bytes!("../assets/shaders/compiled/fragment/custom.frag.spv").to_vec(),
        ShaderStageFlags::FRAGMENT,
        "main",
    );
}



#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// Draw opaque sprites without lighting.
#[derive(Clone, Debug, PartialEq, Derivative)]
#[derivative(Default(bound = ""))]
pub struct DrawCustomDesc;

impl DrawCustomDesc {
    /// Create instance of `DrawFlat2D` render group
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> RenderGroupDesc<B, World> for DrawCustomDesc {
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

        let (pipeline, pipeline_layout) = build_custom_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            false,
            vec![env.raw_layout(), textures.raw_layout()],
        )?;

        Ok(Box::new(DrawCustom::<B> {
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
pub struct DrawCustom<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: FlatEnvironmentSub<B>,
    textures: TextureSub<B>,
    vertex: DynamicVertexBuffer<B, SpriteArgs>,
    sprites: OneLevelBatch<TextureId, SpriteArgs>,
}

impl<B: Backend> RenderGroup<B, World> for DrawCustom<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        world: &World,
    ) -> PrepareResult {

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
            ReadStorage<'_, SpriteRender>,
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
                    let (batch_data, texture) = SpriteArgs::from_data(
                        &tex_storage,
                        &sprite_sheet_storage,
                        &sprite_render,
                        &global,
                        tint,
                    )?;
                    let (tex_id, _) = textures_ref.insert(
                        factory,
                        world,
                        texture,
                        hal::image::Layout::ShaderReadOnlyOptimal,
                    )?;
                    Some((tex_id, batch_data))
                })
                .for_each_group(|tex_id, batch_data| {
                    sprites_ref.insert(tex_id, batch_data.drain(..))
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
        for (&tex, range) in self.sprites.iter() {
            if self.textures.loaded(tex) {
                self.textures.bind(layout, 1, tex, &mut encoder);
                unsafe {
                    encoder.draw(0..4, range);
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


fn build_custom_pipeline<B: Backend>(
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

    let shader_vertex = unsafe { SPRITE_VERTEX.module(factory).unwrap() };
    let shader_fragment = unsafe { SPRITE_FRAGMENT.module(factory).unwrap() };

    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                .with_vertex_desc(&[(SpriteArgs::vertex(), pso::VertexInputRate::Instance(1))])
                .with_input_assembler(pso::InputAssemblerDesc::new(hal::Primitive::TriangleStrip))
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
                        pso::BlendState::PREMULTIPLIED_ALPHA
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


/// A [RenderPlugin] for drawing 2d objects with flat shading.
/// Required to display sprites defined with [SpriteRender] component.
#[derive(Default, Debug)]
pub struct RenderCustom {
    target: Target,
}

impl RenderCustom {
    /// Set target to which 2d sprites will be rendered.
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }
}

impl<B: Backend> RenderPlugin<B> for RenderCustom {
    fn on_build<'a, 'b>(
        &mut self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add(
            SpriteVisibilitySortingSystem::new(),
            "sprite_visibility_system",
            &[],
        );
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        plan.extend_target(self.target, |ctx| {
            ctx.add(RenderOrder::Opaque, DrawCustomDesc::new().builder())?;
            Ok(())
        });
        Ok(())
    }
}