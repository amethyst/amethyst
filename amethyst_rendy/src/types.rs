use amethyst_assets::{Asset, Handle};
use amethyst_core::ecs::DenseVecStorage;
use rendy::hal::Backend;

#[cfg(feature = "dx12")]
pub type DefaultBackend = rendy::dx12::Backend;

#[cfg(feature = "metal")]
pub type DefaultBackend = rendy::metal::Backend;

#[cfg(feature = "vulkan")]
pub type DefaultBackend = rendy::vulkan::Backend;

#[cfg(not(any(feature = "dx12", feature = "metal", feature = "vulkan")))]
pub type DefaultBackend = rendy::empty::Backend;

pub type Buffer<B> = rendy::resource::Buffer<B>;

/// Mesh wrapper.
pub struct Mesh<B: Backend>(pub rendy::mesh::Mesh<B>);

impl<B: Backend> Asset for Mesh<B> {
    const NAME: &'static str = "Mesh";

    type Data = rendy::mesh::MeshBuilder<'static>;

    type HandleStorage = DenseVecStorage<Handle<Self>>;
}

/// Texture wrapper.
pub struct Texture<B: Backend>(pub rendy::texture::Texture<B>);

impl<B: Backend> Asset for Texture<B> {
    const NAME: &'static str = "Mesh";

    type Data = rendy::texture::TextureBuilder<'static>;

    type HandleStorage = DenseVecStorage<Handle<Self>>;
}
