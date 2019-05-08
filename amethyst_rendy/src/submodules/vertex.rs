use crate::{
    rendy::{
        command::RenderPassEncoder,
        factory::Factory,
        hal,
        memory::{MappedRange, Write},
        resource::{Buffer, Escape},
    },
    types::Backend,
    util,
};
use core::{marker::PhantomData, ops::Range};

#[derive(Debug)]
pub struct DynamicVertex<B: Backend, T: 'static> {
    per_image: Vec<PerImageDynamicVertex<B>>,
    marker: PhantomData<T>,
}

#[derive(Debug)]
struct PerImageDynamicVertex<B: Backend> {
    buffer: Option<Escape<Buffer<B>>>,
}

impl<B: Backend, T: 'static> DynamicVertex<B, T> {
    pub fn new() -> Self {
        Self {
            per_image: Vec::new(),
            marker: PhantomData,
        }
    }

    pub fn write<I>(
        &mut self,
        factory: &Factory<B>,
        index: usize,
        max_num_items: u64,
        iter: I,
    ) -> bool
    where
        I: IntoIterator,
        I::Item: AsRef<[T]>,
    {
        if max_num_items == 0 {
            return false;
        }

        let this_image = {
            while self.per_image.len() <= index {
                self.per_image.push(PerImageDynamicVertex::new());
            }
            &mut self.per_image[index]
        };

        let buf_size = max_num_items * core::mem::size_of::<T>() as u64;
        if let Some((allocated, mut mapped)) = this_image.map(factory, 0..buf_size) {
            let mut writer = unsafe { mapped.write::<u8>(factory.device(), 0..buf_size).unwrap() };
            let mut slice = unsafe { writer.slice() };

            iter.into_iter().for_each(|data| {
                let data_slice = util::slice_as_bytes(data.as_ref());
                let tmp = std::mem::replace(&mut slice, &mut []);
                let (dst_slice, rest) = tmp.split_at_mut(data_slice.len());
                dst_slice.copy_from_slice(data_slice);
                std::mem::replace(&mut slice, rest);
            });
            allocated
        } else {
            false
        }
    }

    #[inline]
    pub fn bind(
        &self,
        index: usize,
        binding_id: u32,
        encoder: &mut RenderPassEncoder<'_, B>,
    ) -> bool {
        self.per_image
            .get(index)
            .map_or(false, |i| i.bind(binding_id, encoder))
    }
}

impl<B: Backend> PerImageDynamicVertex<B> {
    fn new() -> Self {
        Self { buffer: None }
    }

    fn ensure(&mut self, factory: &Factory<B>, max_size: u64) -> bool {
        util::ensure_buffer(
            &factory,
            &mut self.buffer,
            hal::buffer::Usage::VERTEX,
            rendy::memory::Dynamic,
            max_size,
        )
        .unwrap()
    }

    fn map<'a>(
        &'a mut self,
        factory: &Factory<B>,
        range: Range<u64>,
    ) -> Option<(bool, MappedRange<'a, B>)> {
        let alloc = self.ensure(factory, range.end);
        if let Some(buffer) = &mut self.buffer {
            Some((alloc, buffer.map(factory.device(), range).unwrap()))
        } else {
            None
        }
    }

    #[inline]
    fn bind(&self, binding_id: u32, encoder: &mut RenderPassEncoder<'_, B>) -> bool {
        if let Some(buffer) = self.buffer.as_ref() {
            encoder.bind_vertex_buffers(binding_id, Some((buffer.raw(), 0)));
            true
        } else {
            false
        }
    }
}
