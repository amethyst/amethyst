
use std::any::Any;
use std::fmt::Debug;

use core::Transform;
use gfx_hal::Backend;
use gfx_hal::buffer::Usage;
use gfx_hal::command::RawCommandBuffer;
use gfx_hal::memory::{Pod, cast_slice};

use memory::{self, Allocator};
use cam::Camera;

error_chain! {
    links {
        Memory(memory::Error, memory::ErrorKind);
    }
}

pub trait IntoUniform<B: Backend>: Debug + Sized {
    type Uniform: Any + Debug + Pod + PartialEq + Send + Sync;
    type Cache: Any + Debug;

    /// Get uniform representation of the value.
    fn into_uniform(&self) -> Self::Uniform;

    /// Create cache
    fn create_cache<A>(allocator: &mut A, device: &B::Device) -> Result<Self::Cache>
    where
        A: Allocator<B>;

    /// Update cached value.
    /// Writes updating command into command buffer
    fn update_cached(&self, cache: &mut Self::Cache, cbuf: &mut B::CommandBuffer);
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

    fn create_cache<A>(allocator: &mut A, device: &B::Device) -> Result<Self::Cache>
    where
        A: Allocator<B>,
    {
        BasicUniformCache::new(allocator, device)
    }

    fn update_cached(&self, cache: &mut Self::Cache, cbuf: &mut B::CommandBuffer) {
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

    fn create_cache<A>(allocator: &mut A, device: &B::Device) -> Result<Self::Cache>
    where
        A: Allocator<B>,
    {
        BasicUniformCache::new(allocator, device)
    }

    fn update_cached(&self, cache: &mut Self::Cache, cbuf: &mut B::CommandBuffer) {
        cache.update(cbuf, self);
    }
}

#[derive(Debug)]
pub struct BasicUniformCache<B: Backend, T: IntoUniform<B>> {
    cached: Option<T::Uniform>,
    buffer: B::Buffer,
}

impl<B, T> BasicUniformCache<B, T>
where
    B: Backend,
    T: IntoUniform<B>,
{
    fn new<A>(allocator: &mut A, device: &B::Device) -> Result<Self>
    where
        A: Allocator<B>,
    {
        use std::mem::{align_of, size_of};

        Ok(BasicUniformCache {
            cached: None,
            buffer: allocator.allocate_buffer(
                device,
                size_of::<T>(),
                align_of::<T>(),
                Usage::UNIFORM,
                None,
            )?,
        })
    }

    fn update(&mut self, cbuf: &mut B::CommandBuffer, value: &T) {
        let value = value.into_uniform();
        if self.cached.map_or(false, |c| c == value) {
            self.cached = Some(value);
            cbuf.update_buffer(&self.buffer, 0, cast_slice(&[value]))
        }
    }
}
