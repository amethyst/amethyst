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

use cirque::{Cirque, Entry};
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

    fn get_cached(&mut self, span: Range<Epoch>) -> (&Buffer<B>, Range<u64>);
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
        let align = 32; //device.get_limits().min_uniform_buffer_offset_alignment as u64;
        let stride = Self::stride(align);
        let count = span.end - span.start;
        let buffer = Self::allocate(align, count as usize, allocator, device)?;

        let mut cache = BasicUniformCache {
            align,
            cached: value,
            buffer,
            offsets: Cirque::from_iter((0..count).map(|i| i * stride)),
        };

        cache.update_from_cached(span, cbuf, allocator, device)?;
        Ok(cache)
    }

    fn update<'a, C>(
        &'a mut self,
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

    fn get_cached(&mut self, span: Range<Epoch>) -> (&Buffer<B>, Range<u64>) {
        use std::mem::size_of;
        let offset = *self.offsets.get(span).occupied().unwrap();
        let range = offset..(offset + size_of::<T>() as u64);
        (&self.buffer, range)
    }
}

impl<B, T> BasicUniformCache<B, T>
where
    B: Backend,
    T: Pod + PartialEq,
{
    fn update_from_cached<'a, C>(
        &'a mut self,
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

        let offset = *match self.offsets.get(span.clone()) {
            Entry::Vacant(vacant) => {
                let count = span.end - span.start;
                let align = self.align;
                let new = Self::allocate(align, count as usize, allocator, device)?;
                let old = replace(buffer, new);
                allocator.destroy_buffer(old);
                let stride = Self::stride(align);
                unsafe {
                    // Safe since there are just offsets.
                    // Real buffer is in deallocation queue.
                    vacant.replace((0..count).map(move |i| i as u64 * stride))
                }
            },
            Entry::Occupied(occupied) => occupied,
        };

        if buffer.visible() {
            let mut writer = buffer.write(false, offset, size_of::<T>(), device).unwrap();
            writer.copy_from_slice(cast_slice(&[self.cached]));
        } else {
            cbuf.update_buffer(buffer.raw(), offset, cast_slice(&[self.cached]));
        }

        Eh::make_valid_until(buffer, span.end);
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
            Usage::UNIFORM,
            Properties::DEVICE_LOCAL,
            false,
        )?;

        Ok(buffer)
    }
}
