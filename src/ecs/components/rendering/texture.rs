//! Graphical texture resource.

use asset_manager::{AssetLoader, Assets};
use gfx::Factory;
use gfx::format::{Formatted, SurfaceTyped};

pub use gfx::texture::Kind;
use gfx_device::gfx_types;
use renderer;
use renderer::target::ColorFormat;

/// Handle to a texture resource.
pub type Texture = renderer::Texture<gfx_types::Resources>;

/// Loads raw texture data.
pub struct TextureLoadData<'a> {
    /// Type of storage data being used.
    pub kind: Kind,
    /// Slice of slices with each inner slice representing an image/texture's
    /// pixels laid out contiguously.
    ///
    /// FIXME: Ew, this is gross. Maybe we could use type aliases to make this
    /// a little more readable?
    pub raw: &'a [&'a [<<ColorFormat as Formatted>::Surface as SurfaceTyped>::DataType]],
}

impl<'a> AssetLoader<Texture> for TextureLoadData<'a> {
    /// # Panics
    ///
    /// Panics if factory isn't registered as loader.
    fn from_data(assets: &mut Assets, data: TextureLoadData) -> Option<Texture> {
        let factory = assets
            .get_loader_mut::<gfx_types::Factory>()
            .expect("Couldn't retrieve factory.");
        let tex_res_view =
            match factory.create_texture_immutable::<ColorFormat>(data.kind, data.raw) {
                Ok((_, tex_res_view)) => tex_res_view,
                Err(_) => return None,
            };
        Some(renderer::Texture::Texture(tex_res_view))
    }
}

impl AssetLoader<Texture> for [f32; 4] {
    fn from_data(_: &mut Assets, color: [f32; 4]) -> Option<Texture> {
        Some(renderer::Texture::Constant(color))
    }
}
