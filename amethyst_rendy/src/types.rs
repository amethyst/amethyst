use {
    amethyst_assets::{Asset, Handle},
    amethyst_core::specs::VecStorage,
};

#[cfg(feature = "dx12")]
pub type Backend = rendy::dx12::Backend;

#[cfg(feature = "metal")]
pub type Backend = rendy::metal::Backend;

#[cfg(feature = "vulkan")]
pub type Backend = rendy::vulkan::Backend;

#[cfg(not(any(feature = "dx12", feature = "metal", feature = "vulkan")))]
pub type Backend = rendy::empty::Backend;

pub type Buffer = rendy::resource::Buffer<Backend>;

/// Mesh wrapper.
pub struct Mesh(pub rendy::mesh::Mesh<Backend>);

impl Asset for Mesh {
    const NAME: &'static str = "Mesh";

    type Data = rendy::mesh::MeshBuilder<'static>;

    type HandleStorage = VecStorage<Handle<Self>>;
}

/// Texture wrapper.
pub struct Texture(pub rendy::texture::Texture<Backend>);

impl Asset for Texture {
    const NAME: &'static str = "Mesh";

    type Data = rendy::texture::TextureBuilder<'static>;

    type HandleStorage = VecStorage<Handle<Self>>;
}
