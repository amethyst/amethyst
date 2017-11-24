
//! Pass tells renderer how to convert inputs to image.
//!
//! # Definition
//! Pass - is a box which get some:
//!
//! * Inputs
//!   * Attachments
//!   * Meshes combined with:
//!     * Sampled images
//!     * Uniform buffers
//!     * Push constants
//! * Outputs
//!  * Attachments
//!  * Results of queries (Let's forget about it for now)
//!
//! But it gets executed on the GPU.
//! So instead of writing it as a function and calling with arguments needed
//! we have to record commands into buffers and send them to the GPU.
//!
//! We want a way to define a box which will record all necessarry commands in declarative fasion.
//! In order to feed this box with data we also need define `World -> [Input]` conversion
//! (in declarative fasion where possible).
//!
//!

use std::fmt::{self, Debug};
use std::marker::PhantomData;

use core::Transform;
use gfx_hal::Backend;
use gfx_hal::format::Format;
use gfx_hal::memory::cast_slice;
use gfx_hal::pso::{DescriptorSetLayoutBinding, GraphicsShaderSet, PipelineStage};
use gfx_hal::queue::capability::{Supports, Transfer};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon_core::current_thread_index;
use specs::{SystemData, World};

use shaders::ShaderManager;
use vertex::VertexFormat;

pub trait Data<'a, B>
where
    B: Backend,
{
    type DrawData: SystemData<'a>;
    type PrepareData: SystemData<'a>;
}


pub trait Pass<B>: for<'a> Data<'a, B> + Debug + Default
where
    B: Backend,
{
    /// Name of the pass
    const NAME: &'static str;

    /// Input attachments format
    const INPUTS: &'static [Format];

    /// Color attachments format
    const COLORS: &'static [Format];

    /// DepthStencil attachment format
    const DEPTH_STENCIL: Option<Format>;

    /// Bindings
    const BINDINGS: &'static [DescriptorSetLayoutBinding];

    /// Vertices format
    const VERTICES: &'static [VertexFormat<'static>];

    /// Load shaders
    fn shaders<'a>(manager: &'a mut ShaderManager<B>, device: &B::Device) -> Result<GraphicsShaderSet<'a, B>, ::shaders::Error>;

    /// This function designed for
    ///
    /// * allocating buffers and textures
    /// * storing caches in `World`
    /// * filling `DescriptorSet`s
    fn prepare<'a>(
        &mut self,
        cbuf: &mut B::CommandBuffer,
        layout: &B::PipelineLayout,
        device: &B::Device,
        data: <Self as Data<'a, B>>::PrepareData,
    );

    /// This function designed for
    ///
    /// * binding `DescriptorSet`s
    /// * recording `Transfer` and `Graphics` commands to `CommandBuffer`
    fn draw<'a>(&mut self, cbuf: &mut B::CommandBuffer, data: <Self as Data<'a, B>>::DrawData);
}

pub trait AnyPass<B>: Debug
where
    B: Backend,
{
    fn maker() -> Box<Fn() -> Box<AnyPass<B>>>
    where
        Self: Default + Sized + 'static,
    {
        Box::new(|| Box::new(Self::default()))
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
    fn shaders<'a>(&self, manager: &'a mut ShaderManager<B>, device: &B::Device) -> Result<GraphicsShaderSet<'a, B>, ::shaders::Error>;

    /// Reflects [`Pass::prepare`] function
    ///
    /// [`Pass::prepare`]: trait.Pass.html#tymethod.prepare
    fn prepare<'a>(
        &mut self,
        cbuf: &mut B::CommandBuffer,
        layout: &B::PipelineLayout,
        device: &B::Device,
        world: &'a World,
    );

    /// Reflects [`Pass::draw`] function
    ///
    /// [`Pass::draw`]: trait.Pass.html#tymethod.draw
    fn draw<'a>(&mut self, cbuf: &mut B::CommandBuffer, world: &'a World);
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
    fn shaders<'a>(&self, manager: &'a mut ShaderManager<B>, device: &B::Device) -> Result<GraphicsShaderSet<'a, B>, ::shaders::Error> {
        P::shaders(manager, device)
    }

    fn prepare<'a>(
        &mut self,
        cbuf: &mut B::CommandBuffer,
        layout: &B::PipelineLayout,
        device: &B::Device,
        world: &'a World,
    ) {
        <P as Pass<B>>::prepare(
            self,
            cbuf,
            layout,
            device,
            <P as Data<'a, B>>::PrepareData::fetch(&world.res, 0),
        );
    }

    fn draw<'a>(&mut self, cbuf: &mut B::CommandBuffer, world: &'a World) {
        <P as Pass<B>>::draw(
            self,
            cbuf,
            <P as Data<'a, B>>::DrawData::fetch(&world.res, 0),
        );
    }
}
