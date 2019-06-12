use amethyst_assets::{Asset, Handle};
use amethyst_core::ecs::DenseVecStorage;
use serde::{Deserialize, Serialize};

pub trait Backend: rendy::hal::Backend {
    fn unwrap_mesh(mesh: &Mesh) -> Option<&rendy::mesh::Mesh<Self>>;
    fn unwrap_texture(texture: &Texture) -> Option<&rendy::texture::Texture<Self>>;
    fn wrap_mesh(mesh: rendy::mesh::Mesh<Self>) -> Mesh;
    fn wrap_texture(texture: rendy::texture::Texture<Self>) -> Texture;
}

macro_rules! impl_backends {
    ($($variant:ident, $feature:literal, $backend:ty;)*) => {


        impl_single_default!($([$feature, $backend]),*);

        static_assertions::assert_cfg!(
            any($(feature = $feature),*),
            concat!("You must specify at least one graphical backend feature: ", stringify!($($feature),* "See the wiki article https://github.com/amethyst/amethyst/wiki/GraphicalBackendError for more details."))
        );

        pub enum BackendVariant {
            $(
                #[cfg(feature = $feature)]
                $variant,
            )*
        }

        /// Mesh wrapper.
        #[derive(Debug)]
        pub enum Mesh {
            $(
                #[cfg(feature = $feature)]
                $variant(rendy::mesh::Mesh<$backend>),
            )*
        }

        /// Texture wrapper.
        #[derive(Debug)]
        pub enum Texture {
            $(
                #[cfg(feature = $feature)]
                $variant(rendy::texture::Texture<$backend>),
            )*
        }

        $(
            #[cfg(feature = $feature)]
            impl Backend for $backend {
                #[inline]
                #[allow(irrefutable_let_patterns)]
                fn unwrap_mesh(mesh: &Mesh) -> Option<&rendy::mesh::Mesh<Self>> {
                    if let Mesh::$variant(inner) = mesh {
                        Some(inner)
                    } else {
                        None
                    }
                }
                #[inline]
                #[allow(irrefutable_let_patterns)]
                fn unwrap_texture(texture: &Texture) -> Option<&rendy::texture::Texture<Self>> {
                    if let Texture::$variant(inner) = texture {
                        Some(inner)
                    } else {
                        None
                    }
                }
                #[inline]
                fn wrap_mesh(mesh: rendy::mesh::Mesh<Self>) -> Mesh {
                    Mesh::$variant(mesh)
                }
                #[inline]
                fn wrap_texture(texture: rendy::texture::Texture<Self>) -> Texture {
                    Texture::$variant(texture)
                }
            }
        )*
    };
}

// Create `DefaultBackend` type alias only when exactly one backend is selected.
macro_rules! impl_single_default {
    ( $([$feature:literal, $backend:ty]),* ) => {
        impl_single_default!(@ (), ($([$feature, $backend])*));
    };
    (@ ($($prev:literal)*), ([$cur:literal, $backend:ty]) ) => {
        #[cfg(all( feature = $cur, not(any($(feature = $prev),*)) ))]
        pub type DefaultBackend = $backend;
    };
    (@ ($($prev:literal)*), ([$cur:literal, $backend:ty] $([$nf:literal, $nb:ty])*) ) => {
        #[cfg(all( feature = $cur, not(any($(feature = $prev,)* $(feature = $nf),*)) ))]
        pub type DefaultBackend = $backend;

        impl_single_default!(@ ($($prev)* $cur), ($([$nf, $nb])*) );
    };
}

impl_backends!(
    // DirectX 12 is currently disabled because of incomplete gfx-hal support for it.
    // It will be re-enabled when it actually works.
    // Dx12, "dx12", rendy::dx12::Backend; 
    Metal, "metal", rendy::metal::Backend;
    Vulkan, "vulkan", rendy::vulkan::Backend;
    Empty, "empty", rendy::empty::Backend;
);

impl Asset for Mesh {
    const NAME: &'static str = "Mesh";
    type Data = MeshData;
    type HandleStorage = DenseVecStorage<Handle<Self>>;
}

impl Asset for Texture {
    const NAME: &'static str = "Mesh";
    type Data = TextureData;
    type HandleStorage = DenseVecStorage<Handle<Self>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshData(
    #[serde(deserialize_with = "deserialize_data")] pub rendy::mesh::MeshBuilder<'static>,
);

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    Ok(rendy::mesh::MeshBuilder::deserialize(deserializer)?.into_owned())
}
