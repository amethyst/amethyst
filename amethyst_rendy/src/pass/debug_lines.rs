//! Debug lines pass

use std::marker::PhantomData;

use crate::{
    camera::{ActiveCamera, Camera},
    hidden::Hidden,
    mtl::{Material, MaterialDefaults},
    skinning::JointTransforms,
    types::{Mesh, Texture},
    visibility::Visibility,
    debug_drawing::{DebugLine, DebugLinesComponent}
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

use shred_derive::SystemData;
use smallvec::{smallvec, SmallVec};
use std::io::Write;

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
        
        Ok(DrawDebugLines {
            per_line: vec![]
        })
    }
}

#[derive(Debug)]
pub struct DrawDebugLines<B: Backend> {
    per_line: Vec<PerLineSegment<B>>
}

impl<B: Backend> SimpleGraphicsPipeline<B, Resources> for DrawDebugLines<B> {
    type Desc = DrawDebugLinesDesc;

    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        set_layouts: &[DescriptorSetLayout<B>],
        index: usize,
        resources: &Resources,
    ) -> PrepareResult {
 
        let DebugLinePassData {
            line_segments,
            ..
        } = DebugLinePassData::fetch(resources);


        ensure_buffer(
            &factory,
            &mut None,
            //&mut self.buffer,
            rendy::resource::buffer::UniformBuffer,
            std::mem::size_of::<DebugLine>() as u64,
        ).unwrap();

        PrepareResult::DrawRecord
    }

    fn draw(
        &mut self,
        layout: &B::PipelineLayout,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        aux: &Resources,
    ) {
        log::trace!("Drawing debug lines");

    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &Resources) {
        unimplemented!()
    }
}

#[derive(SystemData)]
struct DebugLinePassData<'a> {
    line_segments: ReadStorage<'a, DebugLinesComponent>
}

fn ensure_buffer<B: Backend>(
    factory: &Factory<B>,
    buffer: &mut Option<rendy::resource::Buffer<B>>,
    usage: impl rendy::resource::buffer::Usage,
    min_size: u64,
) -> Result<bool, failure::Error> {
    if buffer.as_ref().map(|b| b.size()).unwrap_or(0) < min_size {
        let new_size = min_size.next_power_of_two();
        let new_buffer = factory.create_buffer(512, new_size, usage)?;
        *buffer = Some(new_buffer);
        Ok(true)
    } else {
        Ok(false)
    }
}

#[derive(Debug)]
struct PerLineSegment<B: Backend> {
    environment_buffer: Option<rendy::resource::Buffer<B>>,
    models_buffer: Option<rendy::resource::Buffer<B>>,
    material_buffer: Option<rendy::resource::Buffer<B>>,
}

impl<B: Backend> PerLineSegment<B> {
    fn new() -> Self {
        Self {
            environment_buffer: None,
            models_buffer: None,
            material_buffer: None,
        }
    }
}
