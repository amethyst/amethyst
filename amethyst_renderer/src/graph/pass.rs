//! Pass tells renderer how to convert inputs to image.
//!
//! # Definition
//! Pass is a trait that takes these inputs
//!   * Attachments
//!   * Meshes combined with:
//!     * Sampled images
//!     * Uniform buffers
//!     * Push constants
//! and provides these outputs:
//!  * Attachments
//!  * Results of queries (Not implemented)
//!
//! The pass prepares and sends data to GPU
//!
//! We want a way to define a pass which will record all necessary commands in declarative fashion.
//! In order to feed this pass with data we also need define `World -> [Input]` conversion.
//!

use std::marker::PhantomData;
use std::fmt::Debug;
use std::ops::Range;

use gfx_hal::Backend;
use gfx_hal::command::{CommandBuffer, RenderPassInlineEncoder};
use gfx_hal::format::Format;
use gfx_hal::pso::{DescriptorSetLayoutBinding, GraphicsShaderSet};
use gfx_hal::queue::capability::{Supports, Transfer};

use smallvec::SmallVec;

use specs::{SystemData, World};

use descriptors::DescriptorPool;
use epoch::Epoch;
use memory::Allocator;
use shaders::ShaderManager;
use vertex::VertexFormat;

use graph::build::PassBuilder;
use descriptors::{Layout, Binder, BindingsList};

/// Tag component.
/// Passes should only process entities with `PassTag<Self>`.
/// But there is no restriction.
pub struct PassTag<P>(PhantomData<P>);

impl<P> Default for PassTag<P> {
    fn default() -> Self {
        PassTag(PhantomData)
    }
}

/// Helper trait to declare associated types with lifetime parameter.
pub trait Data<'a, B>
where
    B: Backend,
{
    /// Data fetched from `World` during `draw` phase. It shouldn't write anything.
    /// There is no way to guarantee that.
    /// Just don't put here any `FetchMut` or `WriteStorage`.
    type DrawData: SystemData<'a>;

    /// Data fetched from `World` during `prepare` phase. It can both read and write.
    type PrepareData: SystemData<'a>;
}

pub trait Pass<B>: for<'a> Data<'a, B> + Debug + Default
where
    B: Backend,
{
    /// Name of the pass
    const NAME: &'static str;

    /// Input attachments desired format.
    /// Format of actual attachment may be different.
    /// It may be larger if another consumer expects larger format.
    /// It may be smaller because of hardware limitations.
    const INPUTS: usize;

    /// Number of colors to write
    const COLORS: usize;

    /// Does pass writes into depth buffer?
    const DEPTH: bool;

    /// Does pass uses stencil?
    const STENCIL: bool;

    /// Vertices format
    const VERTICES: &'static [VertexFormat<'static>];

    type Bindings: BindingsList;
    
    /// Fill layout with bindings
    fn layout(Layout<()>) -> Layout<Self::Bindings>;

    /// Build render pass
    fn build() -> PassBuilder<'static, B>
    where
        Self: 'static,
    {
        PassBuilder::new::<Self>()
    }

    /// Load shaders
    ///
    /// This function gets called during `Graph` build process.
    fn shaders<'a>(
        manager: &'a mut ShaderManager<B>,
        device: &B::Device,
    ) -> Result<GraphicsShaderSet<'a, B>, ::shaders::Error>;

    /// This function designed for
    ///
    /// * allocating buffers and textures
    /// * storing caches in `World`
    /// * filling `DescriptorSet`s
    fn prepare<'a, C>(
        &mut self,
        binder: Binder<Self::Bindings>,
        span: Range<Epoch>,
        descriptors: &mut DescriptorPool<B>,
        cbuf: &mut CommandBuffer<B, C>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
        data: <Self as Data<'a, B>>::PrepareData,
    ) where
        C: Supports<Transfer>;

    /// This function designed for
    ///
    /// * binding `DescriptorSet`s
    /// * recording `Graphics` commands to `CommandBuffer`
    fn draw_inline<'a>(
        &mut self,
        span: Range<Epoch>,
        layout: &B::PipelineLayout,
        encoder: RenderPassInlineEncoder<B>,
        data: <Self as Data<'a, B>>::DrawData,
    );

    fn tag() -> PassTag<Self> {
        PassTag::default()
    }
}


/// Object-safe trait that mirrors `Pass` trait.
/// It's implemented for any type that implements `Pass`.
pub trait AnyPass<B>: Debug
where
    B: Backend,
{
    fn maker() -> Box<AnyPass<B>>
    where
        Self: Default + Sized + 'static,
    {
        Box::new(Self::default())
    }

    /// Name of the pass
    fn name(&self) -> &'static str;

    /// Input attachments format
    fn inputs(&self) -> usize;

    /// Colors count
    fn colors(&self) -> usize;

    /// Uses depth?
    fn depth(&self) -> bool;

    /// Uses stencil?
    fn stencil(&self) -> bool;

    /// Bindings
    fn bindings(&self) -> SmallVec<[DescriptorSetLayoutBinding; 64]>;

    /// Vertices format
    fn vertices(&self) -> &'static [VertexFormat<'static>];

    /// Load shaders
    fn shaders<'a>(
        &self,
        manager: &'a mut ShaderManager<B>,
        device: &B::Device,
    ) -> Result<GraphicsShaderSet<'a, B>, ::shaders::Error>;

    /// Reflects [`Pass::prepare`] function
    ///
    /// [`Pass::prepare`]: trait.Pass.html#tymethod.prepare
    fn prepare<'a>(
        &mut self,
        span: Range<Epoch>,
        descriptors: &mut DescriptorPool<B>,
        cbuf: &mut CommandBuffer<B, Transfer>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
        world: &'a World,
    );

    /// Reflects [`Pass::draw_inline`] function
    ///
    /// [`Pass::draw_inline`]: trait.Pass.html#tymethod.draw_inline
    fn draw_inline<'a>(
        &mut self,
        span: Range<Epoch>,
        layout: &B::PipelineLayout,
        encoder: RenderPassInlineEncoder<B>,
        world: &'a World,
    );
}

impl<P, B> AnyPass<B> for P
where
    P: Pass<B>,
    B: Backend,
{
    /// Name of the pass
    fn name(&self) -> &'static str {
        P::NAME
    }

    /// Input attachments format
    fn inputs(&self) -> usize {
        P::INPUTS
    }

    /// Colors count
    fn colors(&self) -> usize {
        P::COLORS
    }

    /// Uses depth?
    fn depth(&self) -> bool {
        P::DEPTH
    }

    /// Uses stencil?
    fn stencil(&self) -> bool {
        P::STENCIL
    }

    /// Bindings
    fn bindings(&self) -> SmallVec<[DescriptorSetLayoutBinding; 64]> {
        Self::layout(Layout::new()).bindings()
    }

    /// Vertices format
    fn vertices(&self) -> &'static [VertexFormat<'static>] {
        P::VERTICES
    }

    /// Load shaders
    fn shaders<'a>(
        &self,
        manager: &'a mut ShaderManager<B>,
        device: &B::Device,
    ) -> Result<GraphicsShaderSet<'a, B>, ::shaders::Error> {
        P::shaders(manager, device)
    }

    fn prepare<'a>(
        &mut self,
        span: Range<Epoch>,
        descriptors: &mut DescriptorPool<B>,
        cbuf: &mut CommandBuffer<B, Transfer>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
        world: &'a World,
    ) {
        <P as Pass<B>>::prepare(
            self,
            Self::layout(Layout::new()).into(),
            span,
            descriptors,
            cbuf,
            allocator,
            device,
            <P as Data<'a, B>>::PrepareData::fetch(&world.res, 0),
        );
    }

    fn draw_inline<'a>(
        &mut self,
        span: Range<Epoch>,
        layout: &B::PipelineLayout,
        encoder: RenderPassInlineEncoder<B>,
        world: &'a World,
    ) {
        <P as Pass<B>>::draw_inline(
            self,
            span,
            layout,
            encoder,
            <P as Data<'a, B>>::DrawData::fetch(&world.res, 0),
        );
    }
}
