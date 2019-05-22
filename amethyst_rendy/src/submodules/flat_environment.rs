use crate::{
    pod::ViewArgs,
    rendy::{command::RenderPassEncoder, factory::Factory},
    submodules::{gather::CameraGatherer, uniform::DynamicUniform},
    types::Backend,
};
use amethyst_core::ecs::Resources;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

#[derive(Debug)]
pub struct FlatEnvironmentSub<B: Backend> {
    uniform: DynamicUniform<B, ViewArgs>,
}

impl<B: Backend> FlatEnvironmentSub<B> {
    pub fn new(factory: &Factory<B>) -> Result<Self, failure::Error> {
        Ok(Self {
            uniform: DynamicUniform::new(factory, rendy::hal::pso::ShaderStageFlags::VERTEX)?,
        })
    }

    pub fn raw_layout(&self) -> &B::DescriptorSetLayout {
        self.uniform.raw_layout()
    }

    pub fn process(&mut self, factory: &Factory<B>, index: usize, res: &Resources) {
        #[cfg(feature = "profiler")]
        profile_scope!("process");
        let projview = CameraGatherer::gather(res).projview;
        self.uniform.write(factory, index, projview);
    }

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
