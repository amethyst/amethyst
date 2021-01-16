use derivative::Derivative;
use glsl_layout::{vec3, Uniform};
use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::{Factory, UploadError},
    graph::{
        render::{PrepareResult, RenderGroup, RenderGroupDesc},
        GraphContext, NodeBuffer, NodeImage,
    },
    hal::{self, device::Device, pso},
    mesh::{AsVertex, Mesh, PosTex},
    shader::Shader,
};
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{
    palette::Srgb,
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    pod::IntoPod,
    shape::Shape,
    submodules::{DynamicUniform, FlatEnvironmentSub},
    system::GraphAuxData,
    types::Backend,
    util,
};

#[derive(Clone, Debug, PartialEq)]
struct SkyboxSettings {
    nadir_color: Srgb,
    zenith_color: Srgb,
}

impl Default for SkyboxSettings {
    fn default() -> Self {
        Self {
            nadir_color: Srgb::new(0.1, 0.3, 0.35),
            zenith_color: Srgb::new(0.75, 1.0, 1.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Uniform)]
pub(crate) struct SkyboxUniform {
    nadir_color: vec3,
    zenith_color: vec3,
}

impl SkyboxSettings {
    pub(crate) fn uniform(&self) -> <SkyboxUniform as Uniform>::Std140 {
        SkyboxUniform {
            nadir_color: self.nadir_color.into_pod(),
            zenith_color: self.zenith_color.into_pod(),
        }
        .std140()
    }
}

/// Describe drawing a skybox around the camera view
#[derive(Clone, Debug, PartialEq, Derivative)]
#[derivative(Default(bound = ""))]
pub struct DrawSkyboxDesc {
    default_settings: SkyboxSettings,
}

impl DrawSkyboxDesc {
    /// Create instance of [DrawSkybox] render group
    pub fn new() -> Self {
        Default::default()
    }

    /// Defines the [SkyboxSettings] colors to initialize for this render group
    pub fn with_colors(nadir_color: Srgb, zenith_color: Srgb) -> Self {
        Self {
            default_settings: SkyboxSettings {
                nadir_color,
                zenith_color,
            },
        }
    }
}

impl<B: Backend> RenderGroupDesc<B, GraphAuxData> for DrawSkyboxDesc {
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        queue: QueueId,
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
        let colors = DynamicUniform::new(factory, pso::ShaderStageFlags::FRAGMENT)?;
        let mesh = Shape::Sphere(16, 16)
            .generate::<Vec<PosTex>>(None)
            .build(queue, factory)
            .map_err(|e| {
                match e {
                    UploadError::Upload(oom) => oom.into(),
                    _ => pso::CreationError::Other,
                }
            })?;

        let (pipeline, pipeline_layout) = build_skybox_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            vec![env.raw_layout(), colors.raw_layout()],
        )?;

        Ok(Box::new(DrawSkybox::<B> {
            pipeline,
            pipeline_layout,
            env,
            colors,
            mesh,
            default_settings: self.default_settings,
        }))
    }
}

/// Draw a skybox around the camera view
#[derive(Debug)]
pub struct DrawSkybox<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: FlatEnvironmentSub<B>,
    colors: DynamicUniform<B, SkyboxUniform>,
    mesh: Mesh<B>,
    default_settings: SkyboxSettings,
}

impl<B: Backend> RenderGroup<B, GraphAuxData> for DrawSkybox<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        aux: &GraphAuxData,
    ) -> PrepareResult {
        #[cfg(feature = "profiler")]
        profile_scope!("prepare");

        let settings = aux
            .resources
            .get::<SkyboxSettings>()
            .map(|s| s.uniform())
            .unwrap_or_else(|| self.default_settings.uniform());

        self.env.process(factory, index, aux.world, aux.resources);
        let changed = self.colors.write(factory, index, settings);

        if changed {
            PrepareResult::DrawRecord
        } else {
            PrepareResult::DrawReuse
        }
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _aux: &GraphAuxData,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("draw");
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, &self.pipeline_layout, 0, &mut encoder);
        self.colors
            .bind(index, &self.pipeline_layout, 1, &mut encoder);
        self.mesh
            .bind(0, &[PosTex::vertex()], &mut encoder)
            .unwrap();
        unsafe {
            encoder.draw(0..self.mesh.len(), 0..1);
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

fn build_skybox_pipeline<B: Backend>(
    factory: &Factory<B>,
    subpass: hal::pass::Subpass<'_, B>,
    framebuffer_width: u32,
    framebuffer_height: u32,
    layouts: Vec<&B::DescriptorSetLayout>,
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), pso::CreationError> {
    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
    }?;

    let shader_vertex = unsafe { super::SKYBOX_VERTEX.module(factory).unwrap() };
    let shader_fragment = unsafe { super::SKYBOX_FRAGMENT.module(factory).unwrap() };

    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                .with_vertex_desc(&[(PosTex::vertex(), pso::VertexInputRate::Vertex)])
                .with_shaders(util::simple_shader_set(
                    &shader_vertex,
                    Some(&shader_fragment),
                ))
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                .with_depth_test(pso::DepthTest {
                    fun: pso::Comparison::GreaterEqual,
                    write: false,
                })
                .with_blend_targets(vec![pso::ColorBlendDesc {
                    mask: pso::ColorMask::ALL,
                    blend: None,
                }]),
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
