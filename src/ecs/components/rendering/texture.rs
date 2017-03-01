//! Graphical texture resource.

pub use gfx::texture::Kind;

use std::fmt::{Debug, Formatter, Error as FormatError};

use gfx::Factory;
use gfx::CombinedError as GfxError;
use gfx_device::gfx_types;

use asset_manager::Asset;
use engine::Context;
use renderer;
use renderer::target::ColorFormat;

type TextureInternal = renderer::Texture<gfx_types::Resources>;

/// Handle to a texture resource.
#[derive(Clone)]
pub struct Texture {
    /// Internal handle of this texture
    pub inner: TextureInternal,
}

/// Loads raw texture data.
pub struct TextureLoadData {
    /// Type of storage data being used.
    pub kind: Kind,
    /// Slice of slices with each inner slice representing an image/texture's
    /// pixels laid out contiguously.
    pub raw: Box<[Box<[[u8; 4]]>]>,
}

/// Texture data, the data format for the
/// texture asset.
pub enum TextureData {
    /// Raw texture data
    Raw(TextureLoadData),
    /// A one color texture
    Constant([f32; 4]),
}

impl Texture {
    /// Convenience function for creating a texture
    /// with only a single color.
    pub fn from_color(color: [f32; 4]) -> Self {
        Texture { inner: renderer::Texture::Constant(color) }
    }
}

impl Asset for Texture {
    type Data = TextureData;
    type Error = GfxError;

    fn category() -> &'static str {
        "textures"
    }

    fn from_data(data: TextureData, context: &mut Context) -> Result<Self, GfxError> {
        let inner = match data {
            TextureData::Raw(x) => {
                // TODO: I feel bad for using unsafe here but it
                // seems to be safe
                let raw: &[&[[u8; 4]]] = unsafe { ::std::mem::transmute(x.raw.as_ref()) };

                context.factory
                    .create_texture_immutable::<ColorFormat>(x.kind, raw)
                    .map(|(_, x)| renderer::Texture::Texture(x))
            }
            TextureData::Constant(x) => Ok(renderer::Texture::Constant(x)),
        };

        inner.map(|x| Texture { inner: x })
    }
}

impl Debug for Texture {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        let (name, value) = match self.inner {
            renderer::Texture::Texture(_) => ("raw", String::from("<>")),
            renderer::Texture::Constant(x) => {
                ("color", format!("(r={}, g={}, b={}, a={})", x[0], x[1], x[2], x[3]))
            }
        };
        f.debug_struct("Texture").field(name, &value).finish()
    }
}
