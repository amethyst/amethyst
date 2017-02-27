//! Graphical texture resource.

pub use gfx::texture::Kind;

use gfx::Factory;
use gfx::CombinedError as GfxError;
use gfx_device::gfx_types;

use asset_manager::Asset;
use engine::Context;
use renderer;
use renderer::target::ColorFormat;

/// Handle to a texture resource.
pub type Texture = renderer::Texture<gfx_types::Resources>;

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

impl Asset for Texture {
    type Data = TextureData;
    type Error = GfxError;

    fn from_data(data: TextureData, context: &mut Context) -> Result<Self, GfxError> {
        match data {
            TextureData::Raw(x) => {
                // TODO: I feel bad for using unsafe here but it
                // seems to be safe
                let raw: &[&[[u8; 4]]] = unsafe { ::std::mem::transmute(x.raw.as_ref()) };

                context.factory
                    .create_texture_immutable::<ColorFormat>(x.kind, raw)
                    .map(|(_, x)| renderer::Texture::Texture(x))
            }
            TextureData::Constant(x) => Ok(renderer::Texture::Constant(x)),
        }
    }
}
