use amethyst::{
	assets::{AssetStorage, Handle, Loader},
	core::{
        ecs::{Join, Read, ReadExpect, ReadStorage, SystemData, World, DispatcherBuilder,Component, DenseVecStorage},
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
			shader::{PathBufShaderInfo, Shader, ShaderKind, SourceLanguage, SpirvShader},
			texture::TextureBuilder,
            hal::pso::ShaderStageFlags,
		},
        pod::ViewArgs,
		submodules::{gather::CameraGatherer,DynamicIndexBuffer, DynamicVertexBuffer,DynamicUniform, TextureId, TextureSub,FlatEnvironmentSub},
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
	winit::Event,prelude::*,
};

use derivative::Derivative;
use std::{
	borrow::Cow,
	path::PathBuf,
	sync::{Arc, Mutex},
};
use amethyst_error::Error;
use glsl_layout::*;

lazy_static::lazy_static! {
	static ref VERTEX_SRC: SpirvShader = PathBufShaderInfo::new(
		PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/assets/shaders/shaders/vertex/custom.vert")),
		ShaderKind::Vertex,
		SourceLanguage::GLSL,
		"main",
	).precompile().unwrap();

	static ref VERTEX: SpirvShader = SpirvShader::new(
		(*VERTEX_SRC).spirv().unwrap().to_vec(),
		(*VERTEX_SRC).stage(),
		"main",
	);

	static ref FRAGMENT_SRC: SpirvShader = PathBufShaderInfo::new(
		PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/assets/shaders/shaders/fragment/custom.frag")),
		ShaderKind::Fragment,
		SourceLanguage::GLSL,
		"main",
	).precompile().unwrap();

	static ref FRAGMENT: SpirvShader = SpirvShader::new(
		(*FRAGMENT_SRC).spirv().unwrap().to_vec(),
		(*FRAGMENT_SRC).stage(),
		"main",
	);

//static ref SHADERS: ShaderSetBuilder = ShaderSetBuilder::default()
//		.with_vertex(&*VERTEX).unwrap()
//		.with_fragment(&*FRAGMENT).unwrap();
}


#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;
use amethyst_rendy::bundle::IntoAction;

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
        world: &World,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, World>>, failure::Error> {

        let env = DynamicUniform::new(factory, pso::ShaderStageFlags::VERTEX)?;
        let vertex = DynamicVertexBuffer::new();

        let (pipeline, pipeline_layout) = build_custom_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            true,
            vec![env.raw_layout()],
        )?;
        //aux.register::<Triangle>();

        Ok(Box::new(DrawCustom::<B> {
            pipeline,
            pipeline_layout,
            env,
            vertex,
            vertex_count: 0,
        }))
    }
}

/// Draws opaque 2D sprites to the screen without lighting.
#[derive(Debug)]
pub struct DrawCustom<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: DynamicUniform<B, CustomUniformArgs>,
    vertex: DynamicVertexBuffer<B, CustomArgs>,
    vertex_count: usize,
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
            triangles,
        ) = <(
            ReadStorage<'_, Triangle>,
        )>::fetch(world);

        //Get our scale value
        let scale = world.read_resource::<CustomUniformArgs>();

        self.env.write(factory, index, scale.std140());
        self.vertex_count =0;

        for triangle in triangles.join(){
            self.vertex_count += 3;
        }
        let mut vertex_data_iter : Vec<CustomArgs>= Vec::new();

        for triangle in triangles.join(){
            vertex_data_iter.extend(triangle.get_args().iter());
        }

        self.vertex.write(factory,index,self.vertex_count as u64, Some(&vertex_data_iter.iter()));

        PrepareResult::DrawRecord
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _world: &World,
    ){



        if self.vertex_count == 0{
            return;
        }

        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, layout, 0, &mut encoder);
        self.vertex.bind(index, 0, 0, &mut encoder);


        unsafe {
            encoder.draw(0..self.vertex_count as u32, 0..1);
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

    let shader_vertex = unsafe { VERTEX.module(factory).unwrap() };
    let shader_fragment = unsafe { FRAGMENT.module(factory).unwrap() };

    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                .with_vertex_desc(&[(CustomArgs::vertex(), pso::VertexInputRate::Vertex)])
                .with_input_assembler(pso::InputAssemblerDesc::new(hal::Primitive::TriangleList))
                .with_shaders(util::simple_shader_set(
                    &shader_vertex,
                    Some(&shader_fragment),
                ))
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                .with_blend_targets(vec![pso::ColorBlendDesc(
                    pso::ColorMask::ALL,
                    pso::BlendState::ALPHA,
                )])
                .with_depth_test(pso::DepthTest::On {
                    fun: pso::Comparison::LessEqual,
                    write: true,
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
        world.register::<Triangle>();
        world.insert(CustomUniformArgs{scale:1.0});
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        plan.extend_target(self.target, |ctx| {
            ctx.add(RenderOrder::Transparent, DrawCustomDesc::new().builder())?;
            Ok(())
        });
        Ok(())
    }
}

/// Sprite Vertex Data
/// ```glsl,ignore
/// vec2 dir_x;
/// vec2 dir_y;
/// vec2 pos;
/// vec2 u_offset;
/// vec2 v_offset;
/// float depth;
/// vec4 tint;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(4))]
struct CustomArgs {
    /// Rotation of the sprite, X-axis
    pub pos: vec2,
    /// Rotation of the sprite, Y-axis
    pub color: vec4,
}

impl AsVertex for CustomArgs {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            (Format::Rg32Sfloat, "pos"),
            (Format::Rgba32Sfloat, "color"),
        ))
    }
}

/// ViewArgs
/// ```glsl,ignore
/// uniform ViewArgs {
///    uniform mat4 proj;
///    uniform mat4 view;
/// };
/// ```
#[derive(Clone, Copy, Debug, AsStd140)]
#[repr(C, align(4))]
pub struct CustomUniformArgs {
    /// Projection matrix
    pub scale: float,
}



/// Component that stores persistent debug lines to be rendered in DebugLinesPass draw pass.
/// The vector can only be cleared manually.
#[derive(Debug, Default)]
pub struct Triangle {
    /// Lines to be rendered
    pub points: [[f32;2];3],
    pub colors: [[f32;4];3],
}

impl Component for Triangle {
    type Storage = DenseVecStorage<Self>;
}

impl Triangle{
    pub fn get_args(&self) -> [CustomArgs;3]
    {
        [
            CustomArgs{pos: self.points[0].into(), color: self.colors[0].into()},
            CustomArgs{pos: self.points[1].into(), color: self.colors[1].into()},
            CustomArgs{pos: self.points[2].into(), color: self.colors[2].into()},
        ]
    }
}