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

use std::fmt::{Debug};

use gfx_hal::Backend;
use gfx_hal::command::{CommandBuffer, RenderPassInlineEncoder};
use gfx_hal::format::Format;
use gfx_hal::pso::{DescriptorSetLayoutBinding, GraphicsShaderSet};
use gfx_hal::queue::capability::{Supports, Transfer};
use specs::{SystemData, World};

use descriptors::Descriptors;
use epoch::{CurrentEpoch, Epoch};
use memory::Allocator;
use shaders::ShaderManager;
use vertex::VertexFormat;

pub mod build;

use self::build::PassBuilder;

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

    /// Input attachments format
    /// TODO: Replace with simple `usize`
    const INPUTS: &'static [Format];

    /// Color attachments format
    /// /// TODO: Replace with simple `usize`
    const COLORS: &'static [Format];

    /// DepthStencil attachment format
    /// /// TODO: Replace with simple `bool`
    const DEPTH_STENCIL: Option<Format>;

    /// Bindings
    const BINDINGS: &'static [DescriptorSetLayoutBinding];

    /// Vertices format
    const VERTICES: &'static [VertexFormat<'static>];

    /// Build render pass
    fn build() -> PassBuilder<'static, B> where Self: 'static {
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
        through: Epoch,
        current: &CurrentEpoch,
        descriptors: &mut Descriptors<B>,
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
        through: Epoch,
        layout: &B::PipelineLayout,
        encoder: RenderPassInlineEncoder<B>,
        data: <Self as Data<'a, B>>::DrawData,
    );
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
    fn inputs(&self) -> &'static [Format];

    /// Color attachments format
    fn colors(&self) -> &'static [Format];

    /// DepthStencil attachment format
    fn depth_stencil(&self) -> Option<Format>;

    /// Bindings
    fn bindings(&self) -> &'static [DescriptorSetLayoutBinding];

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
        through: Epoch,
        current: &CurrentEpoch,
        descriptors: &mut Descriptors<B>,
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
        through: Epoch,
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
    fn inputs(&self) -> &'static [Format] {
        P::INPUTS
    }

    /// Color attachments format
    fn colors(&self) -> &'static [Format] {
        P::COLORS
    }

    /// DepthStencil attachment format
    fn depth_stencil(&self) -> Option<Format> {
        P::DEPTH_STENCIL
    }

    /// Bindings
    fn bindings(&self) -> &'static [DescriptorSetLayoutBinding] {
        P::BINDINGS
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
        through: Epoch,
        current: &CurrentEpoch,
        descriptors: &mut Descriptors<B>,
        cbuf: &mut CommandBuffer<B, Transfer>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
        world: &'a World,
    ) {
        <P as Pass<B>>::prepare(
            self,
            through,
            current,
            descriptors,
            cbuf,
            allocator,
            device,
            <P as Data<'a, B>>::PrepareData::fetch(&world.res, 0),
        );
    }

    fn draw_inline<'a>(
        &mut self,
        through: Epoch,
        layout: &B::PipelineLayout,
        encoder: RenderPassInlineEncoder<B>,
        world: &'a World,
    ) {
        <P as Pass<B>>::draw_inline(
            self,
            through,
            layout,
            encoder,
            <P as Data<'a, B>>::DrawData::fetch(&world.res, 0),
        );
    }
}
