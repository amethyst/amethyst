use std::collections::VecDeque;
use std::ops::{DerefMut, Range};

use gfx_hal::{Backend, Device};
use gfx_hal::buffer::Usage;
use gfx_hal::command::CommandBuffer;
use gfx_hal::memory::{cast_slice, Pod, Properties};
use gfx_hal::queue::{Supports, Transfer};

use specs::{Entity, MaskedStorage, Storage};

use epoch::{CurrentEpoch, Eh, Epoch};
use memory::{shift_for_alignment, Allocator, Buffer, Result};


pub trait UniformCache<B: Backend, T>: Sized {
    fn new<C>(
        value: T,
        through: Epoch,
        current: &CurrentEpoch,
        cbuf: &mut CommandBuffer<B, C>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
    ) -> Result<Self>
    where
        C: Supports<Transfer>;

    fn update<C>(
        &mut self,
        value: T,
        through: Epoch,
        current: &CurrentEpoch,
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
        through: Epoch,
        current: &CurrentEpoch,
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
        through: Epoch,
        current: &CurrentEpoch,
        cbuf: &mut CommandBuffer<B, C>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
    ) -> Result<()>
    where
        C: Supports<Transfer>,
    {
        if let Some(cache) = self.get_mut(entity) {
            return cache.update(value, through, current, cbuf, allocator, device);
        }
        self.insert(
            entity,
            BasicUniformCache::new(value, through, current, cbuf, allocator, device)?,
        );
        Ok(())
    }
}


#[derive(Debug)]
pub struct BasicUniformCache<B: Backend, T> {
    align: u64,
    cached: T,
    buffer: Buffer<B>,
    offsets: VecDeque<(u64, Epoch)>,
}

impl<B, T> UniformCache<B, T> for BasicUniformCache<B, T>
where
    B: Backend,
    T: Pod + PartialEq,
{
    fn new<C>(
        value: T,
        through: Epoch,
        current: &CurrentEpoch,
        cbuf: &mut CommandBuffer<B, C>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
    ) -> Result<Self>
    where
        C: Supports<Transfer>,
    {
        let align = device.get_limits().min_uniform_buffer_offset_alignment as u64;
        let span = through.0 - current.now().0 + 1;
        let (buffer, offsets) = Self::allocate(align, span, allocator, device)?;

        let mut cache = BasicUniformCache {
            align,
            cached: value,
            buffer,
            offsets,
        };

        cache.update_from_cached(through, current, cbuf, allocator, device);
        Ok(cache)
    }

    fn update<C>(
        &mut self,
        value: T,
        through: Epoch,
        current: &CurrentEpoch,
        cbuf: &mut CommandBuffer<B, C>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
    ) -> Result<()>
    where
        C: Supports<Transfer>,
    {
        if self.cached != value {
            self.cached = value;
            self.update_from_cached(through, current, cbuf, allocator, device)
        } else {
            Ok(())
        }
    }

    fn get_cached(&self) -> (&Buffer<B>, Range<u64>) {
        let size = shift_for_alignment(self.align, ::std::mem::size_of::<T>() as u64);
        let offset = self.offsets.back().unwrap();
        (&self.buffer, offset.0..(offset.0 + size))
    }
}

impl<B, T> BasicUniformCache<B, T>
where
    B: Backend,
    T: Pod + PartialEq,
{
    fn update_from_cached<C>(
        &mut self,
        through: Epoch,
        current: &CurrentEpoch,
        cbuf: &mut CommandBuffer<B, C>,
        allocator: &mut Allocator<B>,
        device: &B::Device,
    ) -> Result<()>
    where
        C: Supports<Transfer>,
    {
        use std::mem::{replace, size_of};
        if self.offsets
            .front()
            .map(|&(_, through)| through >= current.now())
            .unwrap_or(true)
        {
            let span = through.0 - current.now().0 + 1;
            assert!((self.offsets.len() as u64) < span);

            let (buffer, offsets) = Self::allocate(self.align, span, allocator, device)?;
            self.offsets = offsets;
            let buffer = replace(&mut self.buffer, buffer);
            allocator.destroy_buffer(buffer);
        }

        let (offset, _) = self.offsets.pop_front().unwrap();
        if self.buffer.visible() {
            let mut writer = device
                .acquire_mapping_writer(self.buffer.raw(), offset..(offset + size_of::<T>() as u64))
                .unwrap();
            writer.copy_from_slice(&[self.cached]);
            device.release_mapping_writer(writer);
        } else {
            cbuf.update_buffer(self.buffer.raw(), offset, cast_slice(&[self.cached]));
        }

        self.offsets.push_back((offset, through));
        Eh::make_valid_through(&self.buffer, through);
        Ok(())
    }

    fn stride(align: u64) -> u64 {
        let size = ::std::mem::size_of::<T>() as u64;
        let stride = shift_for_alignment(align, size);
        stride
    }

    fn allocate(
        align: u64,
        span: u64,
        allocator: &mut Allocator<B>,
        device: &B::Device,
    ) -> Result<(Buffer<B>, VecDeque<(u64, Epoch)>)> {
        let stride = Self::stride(align);
        let buffer = allocator.create_buffer(
            device,
            span * stride,
            span * stride,
            Usage::UNIFORM,
            Properties::DEVICE_LOCAL,
            false,
        )?;

        let offsets = (0..span).map(|i| (i * stride, Epoch::new())).collect();
        Ok((buffer, offsets))
    }
}
