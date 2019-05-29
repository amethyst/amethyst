//! Wrapper and management data structures for providing automatic buffering, resizing and management
//! of rendy vertex buffer types.

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

/// Type alias for a set of dynamic vertex buffer data to be managed. See the documentation
/// for [DynamicVertexData] for implementation details.
pub type DynamicVertexBuffer<B, T> = DynamicVertexData<B, VertexData<T>, T>;
/// Type alias for a set of dynamic index buffer data to be managed. See the documentation
/// for [DynamicVertexData] for implementation details.
pub type DynamicIndexBuffer<B, T> = DynamicVertexData<B, IndexData<T>, T>;

/// Type used to compile-time specify the type of vertex buffer data managed by a  `DynamicVertexData`
#[derive(Debug)]
pub struct IndexData<T>(PhantomData<T>);

/// Type used to compile-time specify the type of vertex buffer data managed by a  `DynamicVertexData`
#[derive(Debug)]
pub struct VertexData<T>(PhantomData<T>);

/// Type trait for allowing type-based implementation details for binding the different buffer types
/// of index and vertex `DynamicVertexData`
pub trait VertexDataBufferType<B: Backend> {
    /// Returns this type implementations `gfx_hal::buffer::Usage`
    fn usage() -> hal::buffer::Usage;

    /// Executes this types specific binding implementation
    fn bind(
        binding_id: u32,
        encoder: &mut RenderPassEncoder<'_, B>,
        buffer: &Option<Escape<Buffer<B>>>,
        offset: u64,
    ) -> bool;
}

impl<B: Backend, T: 'static> VertexDataBufferType<B> for VertexData<T> {
    #[inline]
    fn usage() -> hal::buffer::Usage {
        hal::buffer::Usage::VERTEX
    }

    #[inline]
    fn bind(
        binding_id: u32,
        encoder: &mut RenderPassEncoder<'_, B>,
        buffer: &Option<Escape<Buffer<B>>>,
        offset: u64,
    ) -> bool {
        if let Some(buffer) = buffer.as_ref() {
            encoder.bind_vertex_buffers(binding_id, Some((buffer.raw(), offset)));
            return true;
        }

        false
    }
}

impl<B: Backend> VertexDataBufferType<B> for IndexData<u16> {
    #[inline]
    fn usage() -> hal::buffer::Usage {
        hal::buffer::Usage::INDEX
    }

    #[inline]
    fn bind(
        _: u32,
        encoder: &mut RenderPassEncoder<'_, B>,
        buffer: &Option<Escape<Buffer<B>>>,
        offset: u64,
    ) -> bool {
        if let Some(buffer) = buffer.as_ref() {
            encoder.bind_index_buffer(buffer.raw(), offset, hal::IndexType::U16);
            return true;
        }

        false
    }
}

impl<B: Backend> VertexDataBufferType<B> for IndexData<u32> {
    #[inline]
    fn usage() -> hal::buffer::Usage {
        hal::buffer::Usage::INDEX
    }

    #[inline]
    fn bind(
        _: u32,
        encoder: &mut RenderPassEncoder<'_, B>,
        buffer: &Option<Escape<Buffer<B>>>,
        offset: u64,
    ) -> bool {
        if let Some(buffer) = buffer.as_ref() {
            encoder.bind_index_buffer(buffer.raw(), offset, hal::IndexType::U32);
            return true;
        }

        false
    }
}

/// This structure wraps [PerImageDynamicVertexData], managing multiple instances and providing
/// an easy-to-use interface for having per-image buffers. This is needed because multiple images
/// (frames) can be in flight at any given time, so multiple buffers are needed for the same data.
#[derive(Debug)]
pub struct DynamicVertexData<B: Backend, V: VertexDataBufferType<B>, T: 'static> {
    per_image: Vec<PerImageDynamicVertexData<B, V>>,
    marker: PhantomData<T>,
}

impl<B: Backend, V: VertexDataBufferType<B>, T: 'static> DynamicVertexData<B, V, T> {
    /// Creates an empty, 0-frame `DynamicVertexData`
    pub fn new() -> Self {
        Self {
            per_image: Vec::new(),
            marker: PhantomData,
        }
    }

    /// Write to the allocated rendy buffer for the specified frame index.
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
                self.per_image.push(PerImageDynamicVertexData::new());
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

    /// Bind the allocated rendy buffer for this frame index.
    #[inline]
    pub fn bind(
        &self,
        index: usize,
        binding_id: u32,
        offset: u64,
        encoder: &mut RenderPassEncoder<'_, B>,
    ) -> bool {
        self.per_image
            .get(index)
            .map_or(false, |i| V::bind(binding_id, encoder, &i.buffer, offset))
    }
}

/// an easy-to-use interface for managing, growing and binding a given vertex buffer type. This
/// implementation also leverages the [VertexDataBufferType] trait type for statically dispatching
/// the appropriate binding and allocation functions, preventing hot-path branching.
#[derive(Debug)]
pub struct PerImageDynamicVertexData<B: Backend, V: VertexDataBufferType<B>> {
    buffer: Option<Escape<Buffer<B>>>,
    marker: PhantomData<V>,
}

impl<B: Backend, V: VertexDataBufferType<B>> PerImageDynamicVertexData<B, V> {
    /// Creates an empty, 0-frame `DynamicVertexData`
    fn new() -> Self {
        Self {
            buffer: None,
            marker: PhantomData,
        }
    }

    /// Garuntees that at least max_size bytes of memory is allocated for this buffer
    /// Calls the utility function, [util::ensure_buffer] to dynamically grow the buffer if needed.
    fn ensure(&mut self, factory: &Factory<B>, max_size: u64) -> bool {
        util::ensure_buffer(
            &factory,
            &mut self.buffer,
            V::usage(),
            rendy::memory::Dynamic,
            max_size,
        )
        .unwrap()
    }

    /// Maps the allocated buffer for writing.
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
}
