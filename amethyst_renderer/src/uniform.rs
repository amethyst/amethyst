use std::any::Any;
use std::fmt::Debug;

use core::Transform;
use gfx_hal::Backend;
use gfx_hal::buffer::Usage;
use gfx_hal::command::CommandBuffer;
use gfx_hal::memory::{cast_slice, Pod, Properties};
use gfx_hal::queue::{Supports, Transfer};

use cam::Camera;
use memory::{Allocator, Buffer, Image, Result};

pub trait IntoUniform<B: Backend>: Debug + Sized {
    type Uniform: Any + Debug + Pod + PartialEq + Send + Sync;
    type Cache: Any + Debug;

    /// Get uniform representation of the value.
    fn into_uniform(&self) -> Self::Uniform;

    /// Create cache
    fn create_cache(allocator: &mut Allocator<B>, device: &B::Device) -> Result<Self::Cache>;

    /// Update cached value.
    /// Writes updating command into command buffer
    fn update_cached<C>(&self, cache: &mut Self::Cache, cbuf: &mut CommandBuffer<B, C>)
    where
        C: Supports<Transfer>;
}

pub type UniformCache<B: Backend, T: IntoUniform<B>> = <T as IntoUniform<B>>::Cache;

impl<B> IntoUniform<B> for Transform
where
    B: Backend,
{
    type Uniform = [[f32; 4]; 4];
    type Cache = BasicUniformCache<B, Transform>;

    fn into_uniform(&self) -> [[f32; 4]; 4] {
        self.0.into()
    }

    fn create_cache(allocator: &mut Allocator<B>, device: &B::Device) -> Result<Self::Cache> {
        BasicUniformCache::new(allocator, device)
    }

    fn update_cached<C>(&self, cache: &mut Self::Cache, cbuf: &mut CommandBuffer<B, C>)
    where
        C: Supports<Transfer>,
    {
        cache.update(cbuf, self);
    }
}

impl<B> IntoUniform<B> for Camera
where
    B: Backend,
{
    type Uniform = [[f32; 4]; 4];
    type Cache = BasicUniformCache<B, Camera>;

    fn into_uniform(&self) -> [[f32; 4]; 4] {
        self.proj.into()
    }

    fn create_cache(allocator: &mut Allocator<B>, device: &B::Device) -> Result<Self::Cache> {
        BasicUniformCache::new(allocator, device)
    }

    fn update_cached<C>(&self, cache: &mut Self::Cache, cbuf: &mut CommandBuffer<B, C>)
    where
        C: Supports<Transfer>,
    {
        cache.update(cbuf, self);
    }
}

#[derive(Debug)]
pub struct BasicUniformCache<B: Backend, T: IntoUniform<B>> {
    cached: Option<T::Uniform>,
    buffer: Buffer<B>,
}

impl<B, T> BasicUniformCache<B, T>
where
    B: Backend,
    T: IntoUniform<B>,
{
    fn new(allocator: &mut Allocator<B>, device: &B::Device) -> Result<Self> {
        use std::mem::{align_of, size_of};

        Ok(BasicUniformCache {
            cached: None,
            buffer: allocator.create_buffer(
                device,
                size_of::<T>() as _,
                align_of::<T>() as _,
                Usage::UNIFORM,
                Properties::DEVICE_LOCAL,
                false,
            )?,
        })
    }

    fn update<C>(&mut self, cbuf: &mut CommandBuffer<B, C>, value: &T)
    where
        C: Supports<Transfer>,
    {
        let value = value.into_uniform();
        if self.cached.map_or(false, |c| c == value) {
            self.cached = Some(value);
            cbuf.update_buffer(self.buffer.raw(), 0, cast_slice(&[value]))
        };
    }
}
