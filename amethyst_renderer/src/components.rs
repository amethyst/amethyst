//!
//! Implements `Component` for types in rendering engine.
//! 

use gfx_hal::Backend;
use specs::{Component, DenseVecStorage};

use mesh::Mesh;
use uniform::BasicUniformCache;

impl<B> Component for Mesh<B>
where
    B: Backend,
{
    type Storage = DenseVecStorage<Self>;
}

impl<B, T> Component for BasicUniformCache<B, T>
where
    B: Backend,
    T: Send + Sync + 'static,
{
    type Storage = DenseVecStorage<Self>;
}
