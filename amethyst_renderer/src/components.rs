//!
//! Implements `Component` for types in rendering engine.
//! 

use gfx_hal::Backend;
use specs::{Component, DenseVecStorage, NullStorage};

use descriptors::DescriptorSet;
use mesh::Mesh;
use graph::PassTag;
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


impl<B, P> Component for DescriptorSet<B, P>
where
    B: Backend,
    P: Send + Sync + 'static,
{
    type Storage = DenseVecStorage<Self>;
}

impl<P> Component for PassTag<P>
where
    P: Send + Sync + 'static,
{
    type Storage = NullStorage<Self>;
}