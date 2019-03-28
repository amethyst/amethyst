//! Debug lines pass

use crate::{
    camera::{ActiveCamera, Camera},
    hidden::Hidden,
    mtl::{Material, MaterialDefaults},
    skinning::JointTransforms,
    types::{Mesh, Texture},
    visibility::Visibility
};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    ecs::{Join, Read, ReadExpect, ReadStorage, Resources, SystemData},
    transform::GlobalTransform,
};
use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{PrepareResult, SimpleGraphicsPipeline, SimpleGraphicsPipelineDesc},
        GraphContext, NodeBuffer, NodeImage,
    },
    hal::{
        pso::{
            BlendState, ColorBlendDesc, ColorMask, EntryPoint, GraphicsShaderSet,
            Specialization,
        },
        Backend,
    },
    resource::set::DescriptorSetLayout,
    shader::Shader,
};

#[derive(Clone, Debug, PartialEq)]
pub struct DrawDebugLinesDesc {
    pub line_width: f32
}

impl Default for DrawDebugLinesDesc {
    fn default() -> Self {
        DrawDebugLinesDesc {
            line_width: 1.0 / 400.0
        }
    }
}

impl DrawDebugLinesDesc {
    pub fn new() -> Self {
        Default::default()
    }    
}

impl<B: Backend> SimpleGraphicsPipelineDesc<B, Resources> for DrawDebugLinesDesc {
    type Pipeline = DrawDebugLines<B>;

    fn load_shader_set<'a>(
            &self,
            storage: &'a mut Vec<B::ShaderModule>,
            factory: &mut Factory<B>,
            _aux: &Resources,
        ) -> GraphicsShaderSet<'a, B> {
            storage.clear();

            log::trace!("Loading shader module '{:#?}'", *super::DEBUG_LINES_VERTEX);
            storage.push(super::DEBUG_LINES_VERTEX.module(factory).unwrap());

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
                geometry: None,
                hull: None,
                domain: None,
            }
    }

    fn colors(&self) -> Vec<ColorBlendDesc> {
        vec![ColorBlendDesc(ColorMask::ALL, BlendState::ALPHA)]
    }

    fn input_assembler(&self) -> gfx_hal::pso::InputAssemblerDesc {
        gfx_hal::pso::InputAssemblerDesc {
            primitive: gfx_hal::Primitive::LineList,
            primitive_restart: gfx_hal::pso::PrimitiveRestart::Disabled
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
    ) -> Result<DrawDebugLines<B>, failure::Error> {
        let buffer = factory.create_buffer(1, 1024, rendy::resource::buffer::UniformBuffer)?;
        Ok(DrawDebugLines {
            buffer,
        })
    }
}

#[derive(Debug)]
pub struct DrawDebugLines<B: Backend> {
    buffer: rendy::resource::Buffer<B>
}

impl<B: Backend> SimpleGraphicsPipeline<B, Resources> for DrawDebugLines<B> {
    type Desc = DrawDebugLinesDesc;

    fn prepare(
        &mut self,
        _factory: &Factory<B>,
        _queue: QueueId,
        _set_layouts: &[DescriptorSetLayout<B>],
        _index: usize,
        resources: &Resources,
    ) -> PrepareResult {
 let (
    active_camera,
    cameras,
    mesh_storage,
    texture_storage,
    material_defaults,
    visibility,
    hiddens,
    meshes,
    materials,
    globals,
    joints,
    ) = <(
    Option<Read<'_, ActiveCamera>>,
    ReadStorage<'_, Camera>,
    Read<'_, AssetStorage<Mesh<B>>>,
    Read<'_, AssetStorage<Texture<B>>>,
    ReadExpect<'_, MaterialDefaults<B>>,
    Option<Read<'_, Visibility>>,
    ReadStorage<'_, Hidden>,
    ReadStorage<'_, Handle<Mesh<B>>>,
    ReadStorage<'_, Handle<Material<B>>>,
    ReadStorage<'_, GlobalTransform>,
    ReadStorage<'_, JointTransforms>,
    ) as SystemData>::fetch(resources);

    let defcam = Camera::standard_2d();
    let identity = GlobalTransform::default();
    let camera = active_camera
        .and_then(|ac| {
            cameras
                .get(ac.entity)
                .map(|camera| (camera, globals.get(ac.entity).unwrap_or(&identity)))
        })
        .unwrap_or_else(|| {
            (&cameras, &globals)
                .join()
                .next()
                .unwrap_or((&defcam, &identity))
        });

        unimplemented!()
    }

    fn draw(
        &mut self,
        _layout: &B::PipelineLayout,
        _encoder: RenderPassEncoder<'_, B>,
        _index: usize,
        _aux: &Resources,
    ) {
        log::trace!("Drawing debug lines");
        unimplemented!()
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &Resources) {
        unimplemented!()
    }
}
