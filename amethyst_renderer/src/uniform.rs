//!
//! Semi-automatic tracking of uniform caches and updates.
//! 

use std::ops::{DerefMut, Range};

use gfx_hal::{Backend, Device};
use gfx_hal::buffer::Usage;
use gfx_hal::command::CommandBuffer;
use gfx_hal::memory::{cast_slice, Pod, Properties};
use gfx_hal::queue::{Supports, Transfer};

use specs::{Entity, MaskedStorage, Storage};

use cirque::Cirque;
use epoch::{Eh, Epoch};
use memory::{shift_for_alignment, Allocator, Buffer, Result};


pub trait UniformCache<B: Backend, T>: Sized {
    fn new<C>(
        value: T,
        span: Range<Epoch>,
        cbuf: &mut CommandBuffer<B, C>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
    ) -> Result<Self>
    where
        C: Supports<Transfer>;

    fn update<C>(
        &mut self,
        value: T,
        span: Range<Epoch>,
        cbuf: &mut CommandBuffer<B, C>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
    ) -> Result<()>
    where
        C: Supports<Transfer>;
        
        fn get_cached(&self) -> (&Buffer<B>, Range<u64>);
}

pub trait UniformCacheStorage<B: Backend, T> {
    fn update_cache<C>(
        &mut self,
        entity: Entity,
        value: T,
        span: Range<Epoch>,
        cbuf: &mut CommandBuffer<B, C>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
    ) -> Result<()>
    where
        C: Supports<Transfer>;
}

impl<'a, B, T, D> UniformCacheStorage<B, T> for Storage<'a, BasicUniformCache<B, T>, D>
where
    B: Backend,
    D: DerefMut<Target = MaskedStorage<BasicUniformCache<B, T>>>,
    T: Pod + PartialEq + Send + Sync + 'static,
{
    fn update_cache<C>(
        &mut self,
        entity: Entity,
        value: T,
        span: Range<Epoch>,
        cbuf: &mut CommandBuffer<B, C>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
    ) -> Result<()>
    where
        C: Supports<Transfer>,
    {
        if let Some(cache) = self.get_mut(entity) {
            return cache.update(value, span, cbuf, allocator, device);
        }
        self.insert(
            entity,
            BasicUniformCache::new(value, span, cbuf, allocator, device)?,
        );
        Ok(())
    }
}


#[derive(Debug)]
pub struct BasicUniformCache<B: Backend, T> {
    align: u64,
    cached: T,
    buffer: Buffer<B>,
    offsets: Cirque<u64>,
}

impl<B, T> UniformCache<B, T> for BasicUniformCache<B, T>
where
    B: Backend,
    T: Pod + PartialEq,
{
    fn new<C>(
        value: T,
        span: Range<Epoch>,
        cbuf: &mut CommandBuffer<B, C>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
    ) -> Result<Self>
    where
        C: Supports<Transfer>,
    {
        let align = device.get_limits().min_uniform_buffer_offset_alignment as u64;
        let stride = Self::stride(align);
        let count = span.end - span.start;
        let buffer = Self::allocate(align, count as usize, allocator, device)?;

        let mut cache = BasicUniformCache {
            align,
            cached: value,
            buffer,
            offsets: Cirque::create((0..count).map(|i| i * stride)),
        };

        cache.update_from_cached(span, cbuf, allocator, device)?;
        Ok(cache)
    }

    fn update<C>(
        &mut self,
        value: T,
        span: Range<Epoch>,
        cbuf: &mut CommandBuffer<B, C>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
    ) -> Result<()>
    where
        C: Supports<Transfer>,
    {
        if self.cached != value {
            self.cached = value;
            self.update_from_cached(span, cbuf, allocator, device)
        } else {
            Ok(())
        }
    }

    fn get_cached(&self) -> (&Buffer<B>, Range<u64>) {
        let stride = Self::stride(self.align);
        let offset = *self.offsets.last().unwrap();
        (&self.buffer, offset..(offset + stride))
    }
}

impl<B, T> BasicUniformCache<B, T>
where
    B: Backend,
    T: Pod + PartialEq,
{
    fn update_from_cached<C>(
        &mut self,
        span: Range<Epoch>,
        cbuf: &mut CommandBuffer<B, C>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
    ) -> Result<()>
    where
        C: Supports<Transfer>,
    {
        use std::mem::{replace, size_of};
        let ref mut buffer = self.buffer;
        let align = self.align;
        let offset = *self.offsets.get_or_try_replace(span.clone(), |count| -> Result<_> {
            let new = Self::allocate(align, count, allocator, device)?;
            let old = replace(buffer, new);
            allocator.destroy_buffer(old);
            let stride = Self::stride(align);
            Ok((0..count).map(move |i| i as u64 * stride))
        })?;

        if buffer.visible() {
            let mut writer = device
                .acquire_mapping_writer(buffer.raw(), offset..(offset + size_of::<T>() as u64))
                .unwrap();
            writer.copy_from_slice(&[self.cached]);
            device.release_mapping_writer(writer);
        } else {
            cbuf.update_buffer(buffer.raw(), offset, cast_slice(&[self.cached]));
        }

        Eh::make_valid_until(&buffer, span.end);
        Ok(())
    }

    fn stride(align: u64) -> u64 {
        let size = ::std::mem::size_of::<T>() as u64;
        let stride = shift_for_alignment(align, size);
        stride
    }

    fn allocate(
        align: u64,
        span: usize,
        allocator: &mut Allocator<B>,
        device: &B::Device,
    ) -> Result<Buffer<B>> {
        let stride = Self::stride(align);
        let buffer = allocator.create_buffer(
            device,
            span as u64 * stride,
            span as u64 * stride,
            Usage::UNIFORM,
            Properties::DEVICE_LOCAL,
            false,
        )?;

        Ok(buffer)
    }
}
