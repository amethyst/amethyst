//! 'Global' rendering type declarations
use amethyst_assets::{
    erased_serde::private::serde::{
        de, de::SeqAccess, ser::SerializeSeq, Deserializer, Serializer,
    },
    prefab::{
        serde_diff::{ApplyContext, DiffContext},
        SerdeDiff,
    },
    Asset,
};
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

#[cfg(any(
    all(target_os = "macos", not(any(feature = "empty", feature = "vulkan"))),
    all(feature = "metal", not(any(feature = "vulkan", feature = "empty")))
))]
#[doc = "Default backend"]
pub type DefaultBackend = rendy::metal::Backend;

#[cfg(any(
    all(
        not(target_os = "macos"),
        not(any(feature = "empty", feature = "metal"))
    ),
    all(feature = "vulkan", not(any(feature = "metal", feature = "empty")))
))]
#[doc = "Default backend"]
pub type DefaultBackend = rendy::vulkan::Backend;

#[cfg(feature = "empty")]
#[doc = "Default backend"]
pub type DefaultBackend = rendy::empty::Backend;

/// Mesh wrapper.
#[derive(Debug, TypeUuid)]
#[uuid = "3017f6f7-b9fa-4d55-8cc5-27f803592569"]
pub enum Mesh {
    #[cfg(target_os = "macos")]
    #[doc = "Mesh Variant"]
    Metal(rendy::mesh::Mesh<rendy::metal::Backend>),
    #[cfg(all(not(target_os = "macos"), not(feature = "empty")))]
    #[doc = "Mesh Variant"]
    Vulkan(rendy::mesh::Mesh<rendy::vulkan::Backend>),
    #[cfg(feature = "empty")]
    #[doc = "Mesh Variant"]
    Empty(rendy::mesh::Mesh<rendy::empty::Backend>),
}

impl Serialize for Mesh {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        unimplemented!()
    }
}

impl<'ge> Deserialize<'ge> for Mesh {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'ge>>::Error>
    where
        D: Deserializer<'ge>,
    {
        unimplemented!()
    }
}

impl Default for Mesh {
    fn default() -> Self {
        unimplemented!()
    }
}
impl Clone for Mesh {
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl SerdeDiff for Mesh {
    fn diff<'a, S: SerializeSeq>(
        &self,
        ctx: &mut DiffContext<'a, S>,
        other: &Self,
    ) -> Result<bool, <S as SerializeSeq>::Error> {
        unimplemented!()
    }

    fn apply<'de, A>(
        &mut self,
        seq: &mut A,
        ctx: &mut ApplyContext,
    ) -> Result<bool, <A as SeqAccess<'de>>::Error>
    where
        A: de::SeqAccess<'de>,
    {
        unimplemented!()
    }
}

/// Texture wrapper.
#[derive(Debug, TypeUuid)]
#[uuid = "af14628f-c707-4921-9ac1-f6ae42b8ee8e"]
pub enum Texture {
    #[cfg(target_os = "macos")]
    #[doc = "Texture Variant"]
    Metal(rendy::texture::Texture<rendy::metal::Backend>),
    #[cfg(all(not(target_os = "macos"), not(feature = "empty")))]
    #[doc = "Texture Variant"]
    Vulkan(rendy::texture::Texture<rendy::vulkan::Backend>),
    #[cfg(feature = "empty")]
    #[doc = "Texture Variant"]
    Empty(rendy::texture::Texture<rendy::empty::Backend>),
}

#[cfg(target_os = "macos")]
impl Backend for rendy::metal::Backend {
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

#[cfg(all(not(target_os = "macos"), not(feature = "empty")))]
impl Backend for rendy::vulkan::Backend {
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
impl Backend for rendy::empty::Backend {
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
