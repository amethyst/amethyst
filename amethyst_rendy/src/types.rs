//! 'Global' rendering type declarations
use amethyst_assets::Asset;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

use crate::system::{MeshProcessorSystem, TextureProcessorSystem};

/// Extension of the rendy Backend trait.
pub trait Backend: rendy::hal::Backend {
    /// Unwrap a Backend to a rendy `Mesh`
    fn unwrap_mesh(mesh: &Mesh) -> Option<&rendy::mesh::Mesh<Self>>;
    /// Unwrap a Backend to a rendy `Texture`
    fn unwrap_texture(texture: &Texture) -> Option<&rendy::texture::Texture<Self>>;
    /// Wrap a rendy `Mesh` to its Backend generic.
    fn wrap_mesh(mesh: rendy::mesh::Mesh<Self>) -> Mesh;
    /// Wrap a rendy `Texture` to its Backend generic.
    fn wrap_texture(texture: rendy::texture::Texture<Self>) -> Texture;
}

// Create `DefaultBackend` type alias only when exactly one backend is selected.
macro_rules! impl_single_default {
    ( $([$feature:literal, $backend:ty]),* ) => {
        impl_single_default!(@ (), ($([$feature, $backend])*));
    };
    (@ ($($prev:literal)*), ([$cur:literal, $backend:ty]) ) => {
        #[cfg(all( feature = $cur, not(any($(feature = $prev),*)) ))]
        #[doc = "Default backend"]
        pub type DefaultBackend = $backend;
    };
    (@ ($($prev:literal)*), ([$cur:literal, $backend:ty] $([$nf:literal, $nb:ty])*) ) => {
        #[cfg(all( feature = $cur, not(any($(feature = $prev,)* $(feature = $nf),*)) ))]
        #[doc = "Default backend"]
        pub type DefaultBackend = $backend;

        impl_single_default!(@ ($($prev)* $cur), ($([$nf, $nb])*) );
    };
}

impl_single_default!(["metal", Metal], ["vulkan", Vulkan], ["empty", Empty]);

/// Backend wrapper.
#[derive(Debug)]
pub enum BackendVariant {
    #[cfg(feature = "metal")]
    #[doc = "Backend Variant"]
    Metal,
    #[cfg(feature = "vulkan")]
    #[doc = "Backend Variant"]
    Vulkan,
    #[cfg(feature = "empty")]
    #[doc = "Backend Variant"]
    Empty,
}

/// Mesh wrapper.
#[derive(Debug)]
pub enum Mesh {
    #[cfg(feature = "metal")]
    #[doc = "Mesh Variant"]
    Metal(rendy::mesh::Mesh<metal>),
    #[cfg(feature = "vulkan")]
    #[doc = "Mesh Variant"]
    Vulkan(rendy::mesh::Mesh<vulkan>),
    #[cfg(feature = "empty")]
    #[doc = "Mesh Variant"]
    Empty(rendy::mesh::Mesh<empty>),
}

/// Texture wrapper.
#[derive(Debug)]
pub enum Texture {
    #[cfg(feature = "metal")]
    #[doc = "Texture Variant"]
    Metal(rendy::texture::Texture<metal>),
    #[cfg(feature = "vulkan")]
    #[doc = "Texture Variant"]
    Vulkan(rendy::texture::Texture<vulkan>),
    #[cfg(feature = "empty")]
    #[doc = "Texture Variant"]
    Empty(rendy::texture::Texture<empty>),
}

#[cfg(feature = "metal")]
impl Backend for Metal {
    #[inline]
    #[allow(irrefutable_let_patterns)]
    fn unwrap_mesh(mesh: &Mesh) -> Option<&rendy::mesh::Mesh<Self>> {
        if let Mesh::Metal(inner) = mesh {
            Some(inner)
        } else {
            None
        }
    }
    #[inline]
    #[allow(irrefutable_let_patterns)]
    fn unwrap_texture(texture: &Texture) -> Option<&rendy::texture::Texture<Self>> {
        if let Texture::Metal(inner) = texture {
            Some(inner)
        } else {
            None
        }
    }
    #[inline]
    fn wrap_mesh(mesh: rendy::mesh::Mesh<Self>) -> Mesh {
        Mesh::Metal(mesh)
    }
    #[inline]
    fn wrap_texture(texture: rendy::texture::Texture<Self>) -> Texture {
        Texture::Metal(texture)
    }
}

#[cfg(feature = "vulkan")]
impl Backend for Vulkan {
    #[inline]
    #[allow(irrefutable_let_patterns)]
    fn unwrap_mesh(mesh: &Mesh) -> Option<&rendy::mesh::Mesh<Self>> {
        if let Mesh::Vulkan(inner) = mesh {
            Some(inner)
        } else {
            None
        }
    }
    #[inline]
    #[allow(irrefutable_let_patterns)]
    fn unwrap_texture(texture: &Texture) -> Option<&rendy::texture::Texture<Self>> {
        if let Texture::Vulkan(inner) = texture {
            Some(inner)
        } else {
            None
        }
    }
    #[inline]
    fn wrap_mesh(mesh: rendy::mesh::Mesh<Self>) -> Mesh {
        Mesh::Vulkan(mesh)
    }
    #[inline]
    fn wrap_texture(texture: rendy::texture::Texture<Self>) -> Texture {
        Texture::Vulkan(texture)
    }
}

#[cfg(feature = "empty")]
impl Backend for Empty {
    #[inline]
    #[allow(irrefutable_let_patterns)]
    fn unwrap_mesh(mesh: &Mesh) -> Option<&rendy::mesh::Mesh<Self>> {
        if let Mesh::Empty(inner) = mesh {
            Some(inner)
        } else {
            None
        }
    }
    #[inline]
    #[allow(irrefutable_let_patterns)]
    fn unwrap_texture(texture: &Texture) -> Option<&rendy::texture::Texture<Self>> {
        if let Texture::Empty(inner) = texture {
            Some(inner)
        } else {
            None
        }
    }
    #[inline]
    fn wrap_mesh(mesh: rendy::mesh::Mesh<Self>) -> Mesh {
        Mesh::Empty(mesh)
    }
    #[inline]
    fn wrap_texture(texture: rendy::texture::Texture<Self>) -> Texture {
        Texture::Empty(texture)
    }
}

amethyst_assets::register_asset_type!(MeshData => Mesh; MeshProcessorSystem<DefaultBackend>);
amethyst_assets::register_asset_type!(TextureData => Texture; TextureProcessorSystem<DefaultBackend>);

impl Asset for Mesh {
    fn name() -> &'static str {
        "Mesh"
    }
    type Data = MeshData;
}

impl Asset for Texture {
    fn name() -> &'static str {
        "Texture"
    }
    type Data = TextureData;
}

/// Newtype for MeshBuilder prefab usage.
#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "c5870fe0-1733-4fb4-827c-4353f8c6002d"]
pub struct MeshData(
    #[serde(deserialize_with = "deserialize_data")] pub rendy::mesh::MeshBuilder<'static>,
);

/// Newtype for TextureBuilder prefab usage.
#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "25063afd-6cc0-487e-982f-a63fed7d7393"]
pub struct TextureData(pub rendy::texture::TextureBuilder<'static>);

impl From<rendy::mesh::MeshBuilder<'static>> for MeshData {
    fn from(builder: rendy::mesh::MeshBuilder<'static>) -> Self {
        Self(builder)
    }
}

impl From<rendy::texture::TextureBuilder<'static>> for TextureData {
    fn from(builder: rendy::texture::TextureBuilder<'static>) -> Self {
        Self(builder)
    }
}

fn deserialize_data<'de, D>(deserializer: D) -> Result<rendy::mesh::MeshBuilder<'static>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    log::debug!("deserialize_data");
    Ok(rendy::mesh::MeshBuilder::deserialize(deserializer)?.into_owned())
}
