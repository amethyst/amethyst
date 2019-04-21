use crate::{
    batch::GroupIterator,
    camera::{ActiveCamera, Camera},
    hidden::{Hidden, HiddenPropagate},
    pass::util,
    pod::{SpriteArgs, ViewArgs, UiViewArgs, UiArgs},
};
use amethyst_assets::AssetStorage;
use amethyst_core::{
    ecs::{Join, Read, ReadStorage, Resources, SystemData},
    transform::GlobalTransform,
};
use derivative::Derivative;
use fnv::FnvHashMap;
use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{
            Layout, PrepareResult, SetLayout, SimpleGraphicsPipeline, SimpleGraphicsPipelineDesc,
        },
        GraphContext, NodeBuffer, NodeImage,
    },
    hal::{
        adapter::PhysicalDevice,
        buffer::Usage as BufferUsage,
        device::Device,
        format::Format,
        pso::{
            BlendState, ColorBlendDesc, ColorMask, DepthStencilDesc, Descriptor,
            DescriptorSetLayoutBinding, DescriptorSetWrite, DescriptorType, ElemStride, Element,
            EntryPoint, GraphicsShaderSet, InstanceRate, ShaderStageFlags, Specialization,
        },
        Backend,
    },
    memory::Write,
    mesh::AsVertex,
    resource::{Buffer, DescriptorSet, DescriptorSetLayout, Escape, Handle as RendyHandle},
    shader::Shader,
};
use smallvec::SmallVec;
use std::collections::hash_map::Entry;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawUiDesc;

impl DrawUiDesc {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> SimpleGraphicsPipelineDesc<B, Resources> for DrawUiDesc {
    type Pipeline = DrawUi<B>;

    fn load_shader_set<'a>(
        &self,
        storage: &'a mut Vec<B::ShaderModule>,
        factory: &mut Factory<B>,
        _aux: &Resources,
    ) -> GraphicsShaderSet<'a, B> {
        storage.clear();

        log::trace!("Loading UI shader '{:#?}'", *super::UI_VERTEX);
        storage.push(unsafe { super::UI_VERTEX.module(factory).unwrap() });

        log::trace!("Loading UI shader '{:#?}'", *super::UI_FRAGMENT);
        storage.push(unsafe { super::UI_FRAGMENT.module(factory).unwrap() });

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
        // TODO(happens): transparency color
        vec![ColorBlendDesc(ColorMask::ALL, BlendState::ALPHA)]
    }

    fn depth_stencil(&self) -> Option<DepthStencilDesc> {
        // TODO(happens): transparency stencil
        None
    }

    fn vertices(&self) -> Vec<(Vec<Element<Format>>, ElemStride, InstanceRate)> {
        vec![UiArgs::VERTEX.gfx_vertex_input_desc(0)]
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
                        ty: DescriptorType::CombinedImageSampler,
                        count: 1,
                        stage_flags: ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    }],
                },
                SetLayout {
                    bindings: vec![DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: DescriptorType::UniformBuffer,
                        count: 1,
                        stage_flags: ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    }],
                },
            ],
            push_constants: vec![(ShaderStageFlags::FRAGMENT, 0..2)],
        }
    }

    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _resource: &Resources,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
        _set_layouts: &[RendyHandle<DescriptorSetLayout<B>>],
    ) -> Result<Self::Pipeline, failure::Error> {
        let ubo_offset_align = factory
            .physical()
            .limits()
            .min_uniform_buffer_offset_alignment;

        Ok(DrawUi { ubo_offset_align, ..Default::default() })
    }
}

#[derive(Debug, Derivative)]
#[derivative(Default(bound = ""))]
pub struct DrawUi<B: Backend> {
    ubo_offset_align: u64,
    _phantom: std::marker::PhantomData<B>,
}

impl<B: Backend> SimpleGraphicsPipeline<B, Resources> for DrawUi<B> {
    type Desc = DrawUiDesc;

    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        set_layouts: &[RendyHandle<DescriptorSetLayout<B>>],
        index: usize,
        resources: &Resources,
    ) -> PrepareResult {
        PrepareResult::DrawReuse
    }

    fn draw(
        &mut self,
        layout: &B::PipelineLayout,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _resources: &Resources,
    ) {}

    fn dispose(self, _factory: &mut Factory<B>, _aux: &Resources) {}
}
