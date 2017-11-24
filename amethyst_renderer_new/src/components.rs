use gfx_hal::Backend;
use specs::{Component, DenseVecStorage};

use mesh::Mesh;
use uniform::{BasicUniformCache, IntoUniform};

impl<B> Component for Mesh<B>
where
    B: Backend,
{
    type Storage = DenseVecStorage<Self>;
}

impl<B, T> Component for BasicUniformCache<B, T>
where
    B: Backend,
    T: IntoUniform<B, Cache=Self> + Component,
{
    type Storage = DenseVecStorage<Self>;
}
