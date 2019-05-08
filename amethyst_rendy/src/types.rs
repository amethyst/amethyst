use amethyst_assets::{Asset, Handle};
use amethyst_core::ecs::DenseVecStorage;
use rendy::hal::Backend;
use serde::{Deserialize, Serialize};

#[cfg(feature = "dx12")]
pub type DefaultBackend = rendy::dx12::Backend;

#[cfg(feature = "metal")]
pub type DefaultBackend = rendy::metal::Backend;

#[cfg(feature = "vulkan")]
pub type DefaultBackend = rendy::vulkan::Backend;

#[cfg(not(any(feature = "dx12", feature = "metal", feature = "vulkan")))]
pub type DefaultBackend = rendy::empty::Backend;

pub type Buffer<B> = rendy::resource::Buffer<B>;

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct MeshData(
    #[serde(deserialize_with = "deserialize_data")] pub rendy::mesh::MeshBuilder<'static>,
);

fn deserialize_data<'de, D>(deserializer: D) -> Result<rendy::mesh::MeshBuilder<'static>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    Ok(rendy::mesh::MeshBuilder::deserialize(deserializer)?.into_owned())
}

impl From<rendy::mesh::MeshBuilder<'static>> for MeshData {
    fn from(builder: rendy::mesh::MeshBuilder<'static>) -> Self {
        Self(builder)
    }
}

/// Mesh wrapper.
pub struct Mesh<B: Backend>(pub rendy::mesh::Mesh<B>);

impl<B: Backend> Asset for Mesh<B> {
    const NAME: &'static str = "Mesh";

    type Data = MeshData;

    type HandleStorage = DenseVecStorage<Handle<Self>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureData(pub rendy::texture::TextureBuilder<'static>);

impl From<rendy::texture::TextureBuilder<'static>> for TextureData {
    fn from(builder: rendy::texture::TextureBuilder<'static>) -> Self {
        Self(builder)
    }
}

/// Texture wrapper.
pub struct Texture<B: Backend>(pub rendy::texture::Texture<B>);

impl<B: Backend> Asset for Texture<B> {
    const NAME: &'static str = "Mesh";

    type Data = TextureData;

    type HandleStorage = DenseVecStorage<Handle<Self>>;
}
