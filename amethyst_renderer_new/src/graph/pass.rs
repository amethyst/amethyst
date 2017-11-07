
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

use std::fmt::Debug;

use core::Transform;
use gfx_hal::Backend;
use gfx_hal::memory::cast_slice;
use gfx_hal::pso::{GraphicsShaderSet, PipelineStage};
use gfx_hal::queue::capability::{Supports, Transfer};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon_core::current_thread_index;
use specs::{SystemData, World};

use uniform::UniformFormat;

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
    fn build() -> B::PipelineLayout;

    /// This function designed for
    ///
    /// * allocating buffers and textures
    /// * storing caches in `World`
    /// * filling `DescriptorSet`s
    fn prepare<'a>(
        &mut self,
        cbuf: &mut B::CommandBuffer,
        layout: &B::PipelineLayout,
        device: &mut B::Device,
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
    /// Reflects [`Pass::prepare`] function
    ///
    /// [`Pass::prepare`]: trait.Pass.html#tymethod.prepare
    fn prepare<'a>(
        &mut self,
        cbuf: &mut B::CommandBuffer,
        layout: &B::PipelineLayout,
        device: &mut B::Device,
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
        device: &mut B::Device,
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
