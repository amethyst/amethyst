

use core::Transform;
use gfx_hal::Backend;
use gfx_hal::command::RawCommandBuffer;
use gfx_hal::memory::{Pod, cast_slice};

use cam::Camera;

error_chain!{

}

pub trait IntoUniform<B: Backend>: Sized {
    type Uniform: Pod + PartialEq;
    type Cache;

    /// Get uniform representation of the value.
    fn into_uniform(&self) -> Self::Uniform;

    /// Create cache
    fn create_cache<A>(allocator: &mut A, device: &mut B::Device) -> Result<Self::Cache>
    where
        A: Allocator;

    /// Update cached value.
    /// Writes updating command into command buffer
    fn update_cached(&self, cache: &mut Self::Cache, cbuf: &mut B::CommandBuffer);
}

impl<'a, B, T> IntoUniform<B> for &'a T
where
    B: Backend,
    T: IntoUniform<B>,
{
    type Uniform = T::Uniform;
    type Cache = T::Cache;

    fn into_uniform(&self) -> T::Uniform {
        T::into_uniform(*self)
    }

    fn create_cache<A>(allocator: &mut A, device: &mut B::Device) -> Result<Self::Cache>
    where
        A: Allocator
    {
        let buffer = allocator.allocate_buffer(device, size_of::<Self::Uniform>(), align_of::<Self::Uniform>(), Usage::UNIFORM)?;
        
    }

    fn update_cached(&self, cache: &mut Self::Cache, cbuf: &mut B::CommandBuffer) {
        T::update_cached(*self, cache, cbuf)
    }
}

impl<'a, B, T> IntoUniform<B> for &'a mut T
where
    B: Backend,
    T: IntoUniform<B>,
{
    type Uniform = T::Uniform;
    type Cache = T::Cache;

    fn into_uniform(&self) -> T::Uniform {
        T::into_uniform(*self)
    }

    fn update_cached(&self, cache: &mut Self::Cache, cbuf: &mut B::CommandBuffer) {
        T::update_cached(*self, cache, cbuf)
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

    fn update_cached(&self, cache: &mut Self::Cache, cbuf: &mut B::CommandBuffer) {
        cache.update(cbuf, self);
    }
}

impl<B> IntoUniform<B> for Transform
where
    B: Backend,
{
    type Uniform = [[f32; 4]; 4];
    type Cache = BasicUniformCache<B, Transform>;

    fn into_uniform(&self) -> [[f32; 4]; 4] {
        (*self).into()
    }

    fn update_cached(&self, cache: &mut Self::Cache, cbuf: &mut B::CommandBuffer) {
        cache.update(cbuf, self);
    }
}

pub struct BasicUniformCache<B: Backend, T: IntoUniform<B>> {
    cached: T::Uniform,
    buffer: B::Buffer,
}

impl<B, T> BasicUniformCache<B, T>
where
    B: Backend,
    T: IntoUniform<B>,
{
    fn update(&mut self, cbuf: &mut B::CommandBuffer, value: &T) {
        let value = value.into_uniform();
        if value != self.cached {
            self.cached = value;
            cbuf.update_buffer(&self.buffer, 0, cast_slice(&[value]))
        }
    }
}
