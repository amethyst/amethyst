//! Environment submodule for shared environmental descriptor set data.
//! Fetches and sets projection set information for a flat pass.
use amethyst_core::ecs::*;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{
    pod::ViewArgs,
    rendy::{command::RenderPassEncoder, factory::Factory},
    submodules::{gather::CameraGatherer, uniform::DynamicUniform},
    types::Backend,
};

/// Submodule for loading and binding descriptor sets for a flat, unlit environment.
/// This also abstracts away the need for handling multiple images in flight, as it provides
/// per-image submissions.
#[derive(Debug)]
pub struct FlatEnvironmentSub<B: Backend> {
    uniform: DynamicUniform<B, ViewArgs>,
}

impl<B: Backend> FlatEnvironmentSub<B> {
    /// Create and allocate a new `EnvironmentSub` with the provided rendy `Factory`
    pub fn new(factory: &Factory<B>) -> Result<Self, rendy::hal::pso::CreationError> {
        Ok(Self {
            uniform: DynamicUniform::new(factory, rendy::hal::pso::ShaderStageFlags::VERTEX)?,
        })
    }

    /// Returns the raw `DescriptorSetLayout` for this environment
    pub fn raw_layout(&self) -> &B::DescriptorSetLayout {
        self.uniform.raw_layout()
    }

    /// Performs any re-allocation and GPU memory writing required for this environment set.
    pub fn process(
        &mut self,
        factory: &Factory<B>,
        index: usize,
        world: &World,
        resources: &Resources,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("process");
        let projview = CameraGatherer::gather(world, resources).projview;
        self.uniform.write(factory, index, projview);
    }

    /// Binds this environment set for all images.
    #[inline]
    pub fn bind(
        &self,
        index: usize,
        pipeline_layout: &B::PipelineLayout,
        set_id: u32,
        encoder: &mut RenderPassEncoder<'_, B>,
    ) {
        self.uniform.bind(index, pipeline_layout, set_id, encoder);
    }
}
