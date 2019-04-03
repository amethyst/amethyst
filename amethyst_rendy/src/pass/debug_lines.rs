//! Debug lines pass

use std::marker::PhantomData;
use gfx_hal::device::Device;
use gfx_hal::adapter::PhysicalDevice;
use crate::{
    camera::{ActiveCamera, Camera},
    hidden::Hidden,
    mtl::{Material, MaterialDefaults},
    skinning::JointTransforms,
    types::{Mesh, Texture},
    visibility::Visibility,
    debug_drawing::{DebugLine, DebugLinesComponent},
    pod,
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
    mesh::{PosColor, Position, Color},
    hal::{
        pso::{
            BlendState, ColorBlendDesc, ColorMask, EntryPoint, GraphicsShaderSet,
            Specialization, Descriptor, DescriptorSetLayoutBinding, 
            DescriptorSetWrite, DescriptorType
        },
        Backend,
    },
    resource::set::{DescriptorSet, DescriptorSetLayout},
    shader::Shader,
};

use shred_derive::SystemData;
use smallvec::{smallvec, SmallVec};
use std::io::Write;
use super::util::ensure_buffer;
use super::util;

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
        resources: &Resources,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
        _set_layouts: &[DescriptorSetLayout<B>],
    ) -> Result<DrawDebugLines<B>, failure::Error> {
        Ok(DrawDebugLines {
            lines_buffer: None,
            projview_buffer: None,
            collected_vec: vec![],
            objects_set: None,
        })
    }
}

#[derive(Debug)]
pub struct DrawDebugLines<B: Backend> {
    lines_buffer: Option<rendy::resource::Buffer<B>>,
    projview_buffer: Option<rendy::resource::Buffer<B>>,
    collected_vec: Vec<PosColor>,
    objects_set: Option<DescriptorSet<B>>,
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
            active_camera,
            cameras,
            global_transforms,
            ..
        } = DebugLinePassData::fetch(resources);
        self.collected_vec.clear();
        for component in (line_segments).join() {
            for line in &component.lines {
                let start_vertex = PosColor{
                    position: line.start,
                    color: line.color
                };
                let end_vertex = PosColor{
                    position: line.end,
                    color: line.color
                };
                self.collected_vec.push(start_vertex);
                self.collected_vec.push(end_vertex);
            }
        }

        let line_buf_size = util::align_size::<pod::ViewArgs>(1, self.collected_vec.len());

        ensure_buffer(
            factory, 
            &mut self.lines_buffer, 
            (gfx_hal::buffer::Usage::VERTEX, rendy::memory::Dynamic), 
            line_buf_size
        ).unwrap();

        let view_args = util::prepare_camera(&active_camera, &cameras, &global_transforms).1;

        if ensure_buffer(
            factory, 
            &mut self.projview_buffer, 
            rendy::resource::buffer::UniformBuffer, 
            util::align_size::<pod::ViewArgs>(1, 1)
        ).unwrap() {
            unsafe {
                let buffer = self.lines_buffer.as_mut().unwrap();
                factory
                    .upload_visible_buffer(buffer, 0, &[view_args])
                    .unwrap();
            }            
        };

        {
        let limits = factory.physical().limits();
        let ubo_offset_align = limits.min_uniform_buffer_offset_alignment;
        let align = ubo_offset_align;
        let obj_set = self.objects_set.get_or_insert_with(|| factory.create_descriptor_set(&set_layouts[0]).unwrap()).raw();
        let projview_size = util::align_size::<pod::ViewArgs>(align, 1);
        let projview_range = Some(0)..Some(projview_size);


        unsafe {
            let buffer = self.lines_buffer.as_mut().unwrap().raw();
            factory.write_descriptor_sets(vec![
                DescriptorSetWrite {
                    set: obj_set,
                    binding: 0,
                    array_offset: 0,
                    descriptors: Some(Descriptor::Buffer(buffer, projview_range.clone())),
                },
            ]);
        }
        }

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
    line_segments: ReadStorage<'a, DebugLinesComponent>,
    active_camera: Option<Read<'a, ActiveCamera>>,
    cameras: ReadStorage<'a, Camera>,
    global_transforms: ReadStorage<'a, GlobalTransform>
}