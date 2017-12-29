
use std::marker::PhantomData;
use std::iter::{Once, Chain, once, empty};
use std::ops::Range;

use gfx_hal::{Backend, Device};
use gfx_hal::pso::{DescriptorPool as RawDescriptorPool, DescriptorRangeDesc, DescriptorSetLayoutBinding, DescriptorType};


use cirque::{Cirque, Entry, EntryMut};
use epoch::Epoch;
use stage::ShaderStage;

const CAPACITY: usize = 1024;


/// Descriptor set tagged by type.
/// So that multiple descriptor sets can be attached to the entity.
pub struct DescriptorSet<B: Backend, P>(Cirque<B::DescriptorSet>, PhantomData<fn() -> P>);
impl<B, P> DescriptorSet<B, P>
where
    B: Backend,
{
    pub fn new() -> Self {
        DescriptorSet(Cirque::new(), PhantomData)
    }

    pub fn get_mut<'a>(&'a mut self, span: Range<Epoch>) -> EntryMut<'a, B::DescriptorSet> {
        self.0.get_mut(span)
    }

    pub fn get<'a>(&'a mut self, span: Range<Epoch>) -> Entry<'a, B::DescriptorSet> {
        self.0.get(span)
    }
}

#[derive(Debug)]
pub struct DescriptorPool<B: Backend> {
    range: Vec<DescriptorRangeDesc>,
    layout: B::DescriptorSetLayout,
    pool: B::DescriptorPool,
    sets: Vec<B::DescriptorSet>,
}

impl<B> DescriptorPool<B>
where
    B: Backend,
{
    pub fn new(bindings: &[DescriptorSetLayoutBinding], device: &B::Device) -> Self {
        let range = bindings_to_range_desc(bindings);
        DescriptorPool {
            layout: device.create_descriptor_set_layout(bindings),
            pool: device.create_descriptor_pool(CAPACITY, &range),
            sets: Vec::new(),
            range,
        }
    }

    pub fn dispose(self, device: &B::Device) {
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

fn bindings_to_range_desc(bindings: &[DescriptorSetLayoutBinding]) -> Vec<DescriptorRangeDesc> {
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

