//!  Helper abstraction for per-image uniform submission.
use core::marker::PhantomData;

use glsl_layout::Uniform;
use rendy::resource::SubRange;

use crate::{
    rendy::{
        command::RenderPassEncoder,
        factory::Factory,
        hal::{self, device::Device},
        memory::{MappedRange, Write},
        resource::{
            Buffer, BufferInfo, DescriptorSet, DescriptorSetLayout, Escape, Handle as RendyHandle,
        },
    },
    types::Backend,
    util::{self},
};

/// Provides per-image abstraction for an arbitrary `DescriptorSet`.
#[derive(Debug)]
pub struct DynamicUniform<B: Backend, T: Uniform>
where
    T::Std140: Sized,
{
    layout: RendyHandle<DescriptorSetLayout<B>>,
    per_image: Vec<PerImageDynamicUniform<B, T>>,
}

#[derive(Debug)]
struct PerImageDynamicUniform<B: Backend, T: Uniform>
where
    T::Std140: Sized,
{
    buffer: Escape<Buffer<B>>,
    set: Escape<DescriptorSet<B>>,
    marker: PhantomData<T>,
}

impl<B: Backend, T: Uniform> DynamicUniform<B, T>
where
    T::Std140: Sized,
{
    /// Create a new `DynamicUniform`, allocating descriptor set memory using the provided `Factory`
    /// Allocate to the supplied shader.
    pub fn new(
        factory: &Factory<B>,
        flags: hal::pso::ShaderStageFlags,
    ) -> Result<Self, hal::pso::CreationError> {
        use rendy::hal::pso::*;

        Ok(Self {
            layout: factory
                .create_descriptor_set_layout(util::set_layout_bindings(Some((
                    1,
                    DescriptorType::Buffer {
                        ty: BufferDescriptorType::Uniform,
                        format: BufferDescriptorFormat::Structured {
                            dynamic_offset: false,
                        },
                    },
                    flags,
                ))))?
                .into(),
            per_image: Vec::new(),
        })
    }

    /// Returns the `DescriptSetLayout` for this set.
    pub fn raw_layout(&self) -> &B::DescriptorSetLayout {
        self.layout.raw()
    }

    /// Write `T` to this descriptor set memory
    pub fn write(&mut self, factory: &Factory<B>, index: usize, item: T::Std140) -> bool {
        let mut changed = false;
        let this_image = {
            while self.per_image.len() <= index {
                self.per_image
                    .push(PerImageDynamicUniform::new(factory, &self.layout));
                changed = true;
            }
            &mut self.per_image[index]
        };

        let mut mapped = this_image.map(factory);
        let mut writer = unsafe {
            mapped
                .write::<u8>(factory.device(), 0..std::mem::size_of::<T::Std140>() as u64)
                .unwrap()
        };
        let slice = unsafe { writer.slice() };

        slice.copy_from_slice(util::slice_as_bytes(&[item]));
        changed
    }

    /// Bind this descriptor set
    #[inline]
    pub fn bind(
        &self,
        index: usize,
        pipeline_layout: &B::PipelineLayout,
        binding_id: u32,
        encoder: &mut RenderPassEncoder<'_, B>,
    ) {
        self.per_image[index].bind(pipeline_layout, binding_id, encoder);
    }
}

impl<B: Backend, T: Uniform> PerImageDynamicUniform<B, T>
where
    T::Std140: Sized,
{
    fn new(factory: &Factory<B>, layout: &RendyHandle<DescriptorSetLayout<B>>) -> Self {
        let buffer = factory
            .create_buffer(
                BufferInfo {
                    size: std::mem::size_of::<T::Std140>() as u64,
                    usage: hal::buffer::Usage::UNIFORM,
                },
                rendy::memory::Dynamic,
            )
            .unwrap();

        let set = factory.create_descriptor_set(layout.clone()).unwrap();
        let desc = hal::pso::Descriptor::Buffer(buffer.raw(), SubRange::WHOLE);
        unsafe {
            let set = set.raw();
            factory.write_descriptor_sets(Some(util::desc_write(set, 0, desc)));
        }
        Self {
            buffer,
            set,
            marker: PhantomData,
        }
    }

    fn map<'a>(&'a mut self, factory: &Factory<B>) -> MappedRange<'a, B> {
        let range = 0..self.buffer.size();
        self.buffer.map(factory.device(), range).unwrap()
    }

    #[inline]
    fn bind(
        &self,
        pipeline_layout: &B::PipelineLayout,
        set_id: u32,
        encoder: &mut RenderPassEncoder<'_, B>,
    ) {
        unsafe {
            encoder.bind_graphics_descriptor_sets(
                pipeline_layout,
                set_id,
                Some(self.set.raw()),
                std::iter::empty(),
            );
        }
    }
}
