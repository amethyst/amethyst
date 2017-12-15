use gfx_hal::{Backend, Device};
use gfx_hal::pso::{DescriptorPool, DescriptorRangeDesc, DescriptorSetLayoutBinding, DescriptorType};
use std::ops::Range;
use std::rc::Weak;

const CAPACITY: usize = 1024;

#[derive(Debug)]
pub struct Descriptors<B: Backend> {
    range: Vec<DescriptorRangeDesc>,
    layout: B::DescriptorSetLayout,
    pool: B::DescriptorPool,
    sets: Vec<B::DescriptorSet>,
}

impl<B> Descriptors<B>
where
    B: Backend,
{
    pub fn new(bindings: &[DescriptorSetLayoutBinding], device: &B::Device) -> Self {
        let range = bindings_to_desc(bindings);
        Descriptors {
            layout: device.create_descriptor_set_layout(bindings),
            pool: device.create_descriptor_pool(CAPACITY, &range),
            sets: Vec::new(),
            range,
        }
    }

    pub fn dispose(mut self, device: &B::Device) {
        #[cfg(feature = "gfx-metal")]
        {
            if device.downcast_ref::<::metal::Device>().is_none() {
                self.pool.reset();
            }
        }
        device.destroy_descriptor_set_layout(self.layout);
    }

    pub fn layout(&self) -> &B::DescriptorSetLayout {
        &self.layout
    }

    pub fn get(&mut self) -> B::DescriptorSet {
        self.sets
            .pop()
            .unwrap_or_else(|| self.pool.allocate_sets(&[&self.layout]).pop().unwrap())
    }

    pub fn put(&mut self, set: B::DescriptorSet) {
        self.sets.push(set);
    }

    pub unsafe fn reset(&mut self) {
        self.pool.reset();
    }
}

fn bindings_to_desc(bindings: &[DescriptorSetLayoutBinding]) -> Vec<DescriptorRangeDesc> {
    let mut desc: Vec<DescriptorRangeDesc> = Vec::new();
    for binding in bindings {
        let desc_len = desc.len();
        desc.extend((desc_len..binding.binding + 1).map(|_| {
            DescriptorRangeDesc {
                ty: DescriptorType::UniformBuffer,
                count: 0,
            }
        }));
        desc[binding.binding].ty = binding.ty;
        desc[binding.binding].count = binding.count;
    }
    desc
}
