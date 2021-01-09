//! Graphics pipeline abstraction
use derivative::Derivative;
use rendy::{
    factory::Factory,
    hal::{
        device::Device,
        pass::Subpass,
        pso::{
            AttributeDesc, BakedStates, BasePipeline, BlendDesc, ColorBlendDesc, CreationError,
            DepthStencilDesc, DepthTest, Face, GraphicsPipelineDesc, GraphicsShaderSet,
            InputAssemblerDesc, Multisampling, PipelineCreationFlags, Primitive, Rasterizer, Rect,
            VertexBufferDesc, VertexInputRate, Viewport,
        },
    },
    mesh::VertexFormat,
};
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{types::Backend, util};

// TODO: make gfx type cloneable
#[derive(Derivative, Debug)]
#[derivative(Clone(bound = ""))]
enum LocalBasePipeline<'a, P> {
    Pipeline(&'a P),
    Index(usize),
    None,
}

/// Builder abstraction for constructing a backend-agnostic rendy `GraphicsPipeline`

#[derive(Derivative, Debug)]
#[derivative(Clone(bound = ""))]
pub struct PipelineDescBuilder<'a, B: Backend> {
    shaders: Option<GraphicsShaderSet<'a, B>>,
    rasterizer: Rasterizer,
    vertex_buffers: Vec<VertexBufferDesc>,
    attributes: Vec<AttributeDesc>,
    input_assembler: InputAssemblerDesc,
    blender: BlendDesc,
    depth_stencil: DepthStencilDesc,
    multisampling: Option<Multisampling>,
    baked_states: BakedStates,
    layout: Option<&'a B::PipelineLayout>,
    subpass: Option<Subpass<'a, B>>,
    flags: PipelineCreationFlags,
    parent: LocalBasePipeline<'a, B::GraphicsPipeline>,
}

impl<'a, B: Backend> PipelineDescBuilder<'a, B> {
    /// Create a new builder instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build with the provided `GraphicsShadersSet`
    pub fn with_shaders(mut self, shaders: GraphicsShaderSet<'a, B>) -> Self {
        self.set_shaders(shaders);
        self
    }
    /// Set to use the provided `GraphicsShaderSet`
    pub fn set_shaders(&mut self, shaders: GraphicsShaderSet<'a, B>) {
        self.shaders.replace(shaders);
    }

    /// Build with the provided `Rasterizer`
    pub fn with_rasterizer(mut self, rasterizer: Rasterizer) -> Self {
        self.set_rasterizer(rasterizer);
        self
    }
    /// Set to use the provided `Rasterizer`
    pub fn set_rasterizer(&mut self, rasterizer: Rasterizer) {
        self.rasterizer = rasterizer;
    }

    /// Build with the provided `VertexBufferDesc` collection
    pub fn with_vertex_buffers(mut self, vertex_buffers: Vec<VertexBufferDesc>) -> Self {
        self.set_vertex_buffers(vertex_buffers);
        self
    }
    /// Set to use the provided `VertexBufferDesc` collection.
    pub fn set_vertex_buffers(&mut self, vertex_buffers: Vec<VertexBufferDesc>) {
        self.vertex_buffers = vertex_buffers;
    }

    /// Build with the provided `AttributeDesc` collection
    pub fn with_attributes(mut self, attributes: Vec<AttributeDesc>) -> Self {
        self.set_attributes(attributes);
        self
    }
    /// Set to use the provided `AttributeDesc` collection.
    pub fn set_attributes(&mut self, attributes: Vec<AttributeDesc>) {
        self.attributes = attributes;
    }

    /// Build with the provided `InputAssemblerDesc`
    pub fn with_input_assembler(mut self, input_assembler: InputAssemblerDesc) -> Self {
        self.set_input_assembler(input_assembler);
        self
    }
    /// Set to use the provided `InputAssemblerDesc`
    pub fn set_input_assembler(&mut self, input_assembler: InputAssemblerDesc) {
        self.input_assembler = input_assembler;
    }

    /// Build with the provided `BlendDesc`
    pub fn with_blender(mut self, blender: BlendDesc) -> Self {
        self.set_blender(blender);
        self
    }
    /// Set to use the provided `BlendDesc`
    pub fn set_blender(&mut self, blender: BlendDesc) {
        self.blender = blender;
    }

    /// Build with the provided `DepthStencilDesc`
    pub fn with_depth_stencil(mut self, depth_stencil: DepthStencilDesc) -> Self {
        self.set_depth_stencil(depth_stencil);
        self
    }
    /// Set to use the provided `DepthStencilDesc`
    pub fn set_depth_stencil(&mut self, depth_stencil: DepthStencilDesc) {
        self.depth_stencil = depth_stencil;
    }

    /// Build with the provided `Multisampling`
    pub fn with_multisampling(mut self, multisampling: Option<Multisampling>) -> Self {
        self.set_multisampling(multisampling);
        self
    }
    /// Set to use the provided `Multisampling`
    pub fn set_multisampling(&mut self, multisampling: Option<Multisampling>) {
        self.multisampling = multisampling;
    }

    /// Build with the provided `BakedStates`
    pub fn with_baked_states(mut self, baked_states: BakedStates) -> Self {
        self.set_baked_states(baked_states);
        self
    }
    /// Set to use the provided `BakedStates`
    pub fn set_baked_states(&mut self, baked_states: BakedStates) {
        self.baked_states = baked_states;
    }

    /// Build with the provided `PipelineLayout`
    pub fn with_layout(mut self, layout: &'a B::PipelineLayout) -> Self {
        self.set_layout(layout);
        self
    }
    /// Set to use the provided `PipelineLayout`
    pub fn set_layout(&mut self, layout: &'a B::PipelineLayout) {
        self.layout.replace(layout);
    }

    /// Build with the provided `Subpass`
    pub fn with_subpass(mut self, subpass: Subpass<'a, B>) -> Self {
        self.set_subpass(subpass);
        self
    }
    /// Set to use the provided `Subpass`
    pub fn set_subpass(&mut self, subpass: Subpass<'a, B>) {
        self.subpass.replace(subpass);
    }

    /// Build with the provided `PipelineCreationFlags`
    pub fn with_flags(mut self, flags: PipelineCreationFlags) -> Self {
        self.set_flags(flags);
        self
    }
    /// Set to use the provided `PipelineCreationFlags`
    pub fn set_flags(&mut self, flags: PipelineCreationFlags) {
        self.flags = flags;
    }

    /// Build with the provided `BasePipeline`
    pub fn with_parent(mut self, parent: BasePipeline<'a, B::GraphicsPipeline>) -> Self {
        self.set_parent(parent);
        self
    }
    /// Set to use the provided `BasePipeline`
    pub fn set_parent(&mut self, parent: BasePipeline<'a, B::GraphicsPipeline>) {
        self.parent = match parent {
            BasePipeline::Pipeline(p) => LocalBasePipeline::Pipeline(p),
            BasePipeline::Index(i) => LocalBasePipeline::Index(i),
            BasePipeline::None => LocalBasePipeline::None,
        };
    }

    /// Build with the provided framebuffer size.
    pub fn with_framebuffer_size(mut self, fb_w: u32, fb_h: u32) -> Self {
        self.set_framebuffer_size(fb_w, fb_h);
        self
    }
    /// Set to use the provided framebuffer size.
    pub fn set_framebuffer_size(&mut self, fb_w: u32, fb_h: u32) {
        let rect = Rect {
            x: 0,
            y: 0,
            w: fb_w as i16,
            h: fb_h as i16,
        };
        let old_baked_states = self.baked_states.clone();
        self.set_baked_states(BakedStates {
            viewport: Some(Viewport {
                rect,
                depth: old_baked_states.viewport.map_or(0.0..1.0, |v| v.depth),
            }),
            scissor: Some(rect),
            ..old_baked_states
        })
    }

    /// Build with the provided `DepthTest`
    pub fn with_depth_test(mut self, depth_test: DepthTest) -> Self {
        self.set_depth_test(depth_test);
        self
    }
    /// Set to use the provided `DepthTest`
    pub fn set_depth_test(&mut self, depth_test: DepthTest) {
        self.depth_stencil.depth = Some(depth_test);
    }

    /// Build with the provided `Face` culling.
    pub fn with_face_culling(mut self, cull_face: Face) -> Self {
        self.set_face_culling(cull_face);
        self
    }
    /// Set to use the provided `Face` culling.
    pub fn set_face_culling(&mut self, cull_face: Face) {
        self.rasterizer.cull_face = cull_face;
    }

    /// Build with the provided vertex description.
    pub fn with_vertex_desc(mut self, desc: &[(VertexFormat, VertexInputRate)]) -> Self {
        self.set_vertex_desc(desc);
        self
    }
    /// Set to use the provided vertex description.
    pub fn set_vertex_desc(&mut self, desc: &[(VertexFormat, VertexInputRate)]) {
        let (vbos, attrs) = util::vertex_desc(desc);
        self.set_vertex_buffers(vbos);
        self.set_attributes(attrs);
    }

    /// Build with the provided `ColorBlendDesc` collection.
    pub fn with_blend_targets(mut self, targets: Vec<ColorBlendDesc>) -> Self {
        self.set_blend_targets(targets);
        self
    }
    /// Set to use the provided `ColorBlendDesc` collection
    pub fn set_blend_targets(&mut self, targets: Vec<ColorBlendDesc>) {
        self.blender.targets = targets;
    }
    /// Finalize and construct the `GraphicsPipelineDesc`
    pub fn build(self) -> GraphicsPipelineDesc<'a, B> {
        GraphicsPipelineDesc {
            shaders: self.shaders.expect("Pipeline is missing shaders"),
            rasterizer: self.rasterizer,
            vertex_buffers: self.vertex_buffers,
            attributes: self.attributes,
            input_assembler: self.input_assembler,
            blender: self.blender,
            depth_stencil: self.depth_stencil,
            multisampling: self.multisampling,
            baked_states: self.baked_states,
            layout: self.layout.expect("Pipeline is missing layout"),
            subpass: self.subpass.expect("Pipeline is missing subpass"),
            flags: self.flags,
            parent: match self.parent {
                LocalBasePipeline::Pipeline(p) => BasePipeline::Pipeline(p),
                LocalBasePipeline::Index(i) => BasePipeline::Index(i),
                LocalBasePipeline::None => BasePipeline::None,
            },
        }
    }
}

impl<'a, B: Backend> Default for PipelineDescBuilder<'a, B> {
    fn default() -> Self {
        Self {
            shaders: None,
            rasterizer: Rasterizer::FILL,
            vertex_buffers: Vec::new(),
            attributes: Vec::new(),
            input_assembler: InputAssemblerDesc::new(Primitive::TriangleList),
            blender: BlendDesc::default(),
            depth_stencil: DepthStencilDesc::default(),
            multisampling: None,
            baked_states: BakedStates::default(),
            layout: None,
            subpass: None,
            flags: PipelineCreationFlags::empty(),
            parent: LocalBasePipeline::None,
        }
    }
}

/// Pipeline builder set.
#[derive(Default, Debug, Clone)]
pub struct PipelinesBuilder<'a, B: Backend> {
    builders: Vec<PipelineDescBuilder<'a, B>>,
}

impl<'a, B: Backend> PipelinesBuilder<'a, B> {
    /// Create a new pipeline builder.
    pub fn new() -> Self {
        Self {
            builders: Vec::new(),
        }
    }

    /// Build with an additional `PipelineDescBuilder` instance.
    pub fn with_pipeline(mut self, builder: PipelineDescBuilder<'a, B>) -> Self {
        self.add_pipeline(builder);
        self
    }

    /// Add an additional `PipelineDescBuilder` instance.
    pub fn add_pipeline(&mut self, builder: PipelineDescBuilder<'a, B>) {
        self.builders.push(builder);
    }

    /// Build with an additional child `PipelineDescBuilder` instance.
    pub fn with_child_pipeline(
        mut self,
        index: usize,
        builder: PipelineDescBuilder<'a, B>,
    ) -> Self {
        self.add_child_pipeline(index, builder);
        self
    }

    /// Add an additional child `PipelineDescBuilder` instance.
    pub fn add_child_pipeline(&mut self, index: usize, builder: PipelineDescBuilder<'a, B>) {
        self.builders[index].flags |= PipelineCreationFlags::ALLOW_DERIVATIVES;
        self.builders
            .push(builder.with_parent(BasePipeline::Index(index)));
    }

    /// Finalize and construct the `GraphicsPipeline`
    pub fn build(
        self,
        factory: &Factory<B>,
        cache: Option<&B::PipelineCache>,
    ) -> Result<Vec<B::GraphicsPipeline>, CreationError> {
        #[cfg(feature = "profiler")]
        profile_scope!("create_pipelines");

        let mut pipelines = unsafe {
            factory
                .device()
                .create_graphics_pipelines(self.builders.into_iter().map(|b| b.build()), cache)
        };

        if let Some(err) = pipelines.iter().find_map(|p| p.as_ref().err().cloned()) {
            for p in pipelines.drain(..).filter_map(Result::ok) {
                unsafe {
                    factory.destroy_graphics_pipeline(p);
                }
            }
            return Err(err);
        }

        Ok(pipelines.into_iter().map(|p| p.unwrap()).collect())
    }
}
