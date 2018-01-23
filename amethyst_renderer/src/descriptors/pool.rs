use std::iter::{empty, once, Chain, Once};
use std::marker::PhantomData;
use std::ops::Range;

use gfx_hal::{Backend, Device};
use gfx_hal::pso::{DescriptorPool as RawDescriptorPool, DescriptorRangeDesc,
                   DescriptorSetLayoutBinding, DescriptorType};

use cirque::{Cirque, Entry, EntryMut};
use epoch::Epoch;
use stage::ShaderStage;

const CAPACITY: usize = 1024 * 32;

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

    pub unsafe fn dispose(mut self, pool: &mut DescriptorPool<B>) {
        for (_, set) in self.0.drain() {
            pool.free(set);
        }
    }
}

#[derive(Debug)]
pub struct DescriptorPool<B: Backend> {
    range: Vec<DescriptorRangeDesc>,
    layout: B::DescriptorSetLayout,
    pools: Vec<B::DescriptorPool>,
    sets: Vec<B::DescriptorSet>,
    count: usize,
}

impl<B> DescriptorPool<B>
where
    B: Backend,
{
    pub fn new(bindings: &[DescriptorSetLayoutBinding], device: &B::Device) -> Self {
        let range = bindings_to_range_desc(bindings);
        DescriptorPool {
            layout: device.create_descriptor_set_layout(bindings),
            pools: Vec::new(),
            sets: Vec::new(),
            range,
            count: 0,
        }
    }

    pub fn dispose(self, device: &B::Device) {
        assert_eq!(self.count, self.sets.len());
        #[cfg(feature = "gfx-metal")]
        {
            if device.downcast_ref::<::metal::Device>().is_none() {
                for pool in self.pools {
                    pool.reset();
                }
            }
        }
        drop(self.sets);
        device.destroy_descriptor_set_layout(self.layout);
    }

    pub fn layout(&self) -> &B::DescriptorSetLayout {
        &self.layout
    }

    pub fn allocate(&mut self, device: &B::Device) -> B::DescriptorSet {
        if self.sets.is_empty() {
            // Check if there is sets available
            if self.count == self.pools.len() * CAPACITY {
                // Allocate new pool
                self.pools
                    .push(device.create_descriptor_pool(CAPACITY, &self.range));
            }
            self.count += 1;
            // allocate set
            self.pools
                .last_mut()
                .unwrap()
                .allocate_sets(&[&self.layout])
                .pop()
                .unwrap()
        } else {
            // get unused set
            self.sets.pop().unwrap()
        }
    }

    pub fn free(&mut self, set: B::DescriptorSet) {
        self.sets.push(set);
    }
}

fn bindings_to_range_desc(bindings: &[DescriptorSetLayoutBinding]) -> Vec<DescriptorRangeDesc> {
    let mut desc: Vec<DescriptorRangeDesc> = Vec::new();
    for binding in bindings {
        let desc_len = desc.len();
        desc.extend(
            (desc_len..binding.binding + 1).map(|_| DescriptorRangeDesc {
                ty: DescriptorType::UniformBuffer,
                count: 0,
            }),
        );
        desc[binding.binding].ty = binding.ty;
        desc[binding.binding].count = binding.count;
    }
    desc
}
