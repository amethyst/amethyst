
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

use vertex::VertexFormat;

error_chain!{}

pub trait Data<'a, B>
where
    B: Backend,
{
    type DrawData: SystemData<'a>;
    type PrepareData: SystemData<'a>;
}


pub trait Pass<B>: for<'a> Data<'a, B> + Debug
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

    fn new() -> Self;

    fn box_new() -> Box<NewAnyPass<B>>
    where
        Self: Sized + 'static,
    {
        Box::new(NewPass::<B, Self>::new())
    }

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

pub trait NewAnyPass<B>: Debug
where
    B: Backend,
{
    fn new_any_pass(&self) -> Box<AnyPass<B>>;
}


pub struct NewPass<B, P>(PhantomData<(B, P)>);
impl<B, P> NewPass<B, P> {
    pub fn new() -> Self {
        NewPass(PhantomData)
    }
}

impl<B, P> Debug for NewPass<B, P>
where
    B: Backend,
    P: Pass<B>,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "NewPass({})", P::NAME)
    }
}

impl<B, P> NewAnyPass<B> for NewPass<B, P>
where
    B: Backend,
    P: Pass<B> + 'static,
{
    fn new_any_pass(&self) -> Box<AnyPass<B>> {
        Box::new(P::new())
    }
}

pub trait AnyPass<B>: Debug
where
    B: Backend,
{
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
