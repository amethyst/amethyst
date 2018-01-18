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
use gfx_hal::device::ShaderError;
use gfx_hal::format::Format;
use gfx_hal::pso::{DescriptorSetLayoutBinding, GraphicsShaderSet};
use gfx_hal::queue::capability::{Supports, Transfer};

use smallvec::SmallVec;

use shred::Resources;
use specs::{Component, DenseVecStorage, NullStorage, SystemData, World};

use descriptors::{DescriptorSet, DescriptorPool};
use epoch::Epoch;
use memory::Allocator;
use vertex::VertexFormat;

use graph::build::PassBuilder;
use descriptors::{Layout, Binder, BindingsList};

/// Tag component.
/// Passes should only process entities with `PassTag<Self>`.
/// But there is no restriction.
pub struct PassTag<P>(PhantomData<fn() -> P>);

impl<P> Component for PassTag<P>
where
    P: Send + Sync + 'static,
{
    type Storage = NullStorage<Self>;
}

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
    /// Data fetched from `World` during draw. It shouldn't write anything.
    /// Except for components that belong to the pass.
    /// There is no way to guarantee that now.
    type PassData: SystemData<'a>;
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
        shaders: &'a mut SmallVec<[B::ShaderModule; 5]>,
        device: &B::Device,
    ) -> Result<GraphicsShaderSet<'a, B>, ShaderError>;

    /// This function designed for
    ///
    /// * binding `DescriptorSet`s
    /// * recording `Graphics` commands to `CommandBuffer`
    fn prepare<'a, C>(
        &mut self,
        span: Range<Epoch>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
        cbuf: &mut CommandBuffer<B, C>,
        data: <Self as Data<'a, B>>::PassData,
    ) where
        C: Supports<Transfer>;

    /// This function designed for
    ///
    /// * binding `DescriptorSet`s
    /// * recording `Graphics` commands to `CommandBuffer`
    fn draw_inline<'a>(
        &mut self,
        span: Range<Epoch>,
        binder: Binder<B, Self::Bindings>,
        pool: &mut DescriptorPool<B>,
        device: &B::Device,
        encoder: RenderPassInlineEncoder<B>,
        data: <Self as Data<'a, B>>::PassData,
    );

    /// Cleanup when this pass is removed from graph.
    fn cleanup(&mut self, pool: &mut DescriptorPool<B>, res: &Resources);

    fn tag() -> PassTag<Self> {
        PassTag::default()
    }

    /// Register whatever is required
    fn register(world: &mut World);
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
        shaders: &'a mut SmallVec<[B::ShaderModule; 5]>,
        device: &B::Device,
    ) -> Result<GraphicsShaderSet<'a, B>, ShaderError>;

    /// Reflects [`Pass::prepare`] function
    ///
    /// [`Pass::prepare`]: trait.Pass.html#tymethod.prepare
    fn prepare<'a>(
        &mut self,
        span: Range<Epoch>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
        cbuf: &mut CommandBuffer<B, Transfer>,
        res: &'a Resources,
    );

    /// Reflects [`Pass::draw_inline`] function
    ///
    /// [`Pass::draw_inline`]: trait.Pass.html#tymethod.draw_inline
    fn draw_inline<'a>(
        &mut self,
        span: Range<Epoch>,
        layout: &B::PipelineLayout,
        pool: &mut DescriptorPool<B>,
        device: &B::Device,
        encoder: RenderPassInlineEncoder<B>,
        res: &'a Resources,
    );

    /// Cleanup when this pass is removed from graph.
    fn cleanup(&mut self, pool: &mut DescriptorPool<B>, res: &Resources);
}

impl<P, B> AnyPass<B> for P
where
    P: Pass<B> + 'static,
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
        shaders: &'a mut SmallVec<[B::ShaderModule; 5]>,
        device: &B::Device,
    ) -> Result<GraphicsShaderSet<'a, B>, ShaderError> {
        P::shaders(shaders, device)
    }

    fn prepare<'a>(
        &mut self,
        span: Range<Epoch>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
        cbuf: &mut CommandBuffer<B, Transfer>,
        res: &'a Resources,
    ) {
        <P as Pass<B>>::prepare(
            self,
            span,
            allocator,
            device,
            cbuf,
            <P as Data<'a, B>>::PassData::fetch(res, 0),
        );
    }

    fn draw_inline<'a>(
        &mut self,
        span: Range<Epoch>,
        layout: &B::PipelineLayout,
        pool: &mut DescriptorPool<B>,
        device: &B::Device,
        encoder: RenderPassInlineEncoder<B>,
        res: &'a Resources,
    ) {
        <P as Pass<B>>::draw_inline(
            self,
            span,
            Binder::<B, P::Bindings>::new(layout, P::layout(Layout::new())),
            pool,
            device,
            encoder,
            <P as Data<'a, B>>::PassData::fetch(res, 0),
        );
    }

    fn cleanup(&mut self, pool: &mut DescriptorPool<B>, res: &Resources) {
        <P as Pass<B>>::cleanup(self, pool, res)
    }
}
