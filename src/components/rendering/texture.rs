extern crate gfx_device_gl;
extern crate gfx;

use renderer;
use renderer::target::ColorFormat;
pub use self::gfx::tex::Kind;
use self::gfx::Factory;
use self::gfx::format::{Formatted, SurfaceTyped};
use asset_manager::{AssetLoader, Assets};
use gfx_device::gfx_types;

pub type Texture = renderer::Texture<gfx_types::Resources>;

/// A struct for creating a new texture from raw data
pub struct TextureLoadData<'a> {
    // Which kind of texture storage will be used
    pub kind: Kind,

    // Slice of slices with each inner slice representing an image/texture's
    // pixels laid out contiguously.
    pub raw: &'a [&'a [<<ColorFormat as Formatted>::Surface as SurfaceTyped>::DataType]],
}

impl<'a> AssetLoader<Texture> for TextureLoadData<'a> {
    // # Panics
    // Panics if factory isn't registered as loader.
    fn from_data(assets: &mut Assets, load_data: TextureLoadData) -> Option<Texture> {
        let factory = assets.get_loader_mut::<gfx_types::Factory>().expect("Couldn't retrieve factory.");
        let shader_resource_view = match factory.create_texture_const::<ColorFormat>(load_data.kind, load_data.raw) {
            Ok((_, shader_resource_view)) => shader_resource_view,
            Err(_) => return None,
        };
        Some(renderer::Texture::Texture(shader_resource_view))
    }
}

impl AssetLoader<Texture> for [f32; 4] {
    fn from_data(_: &mut Assets, color: [f32; 4]) -> Option<Texture> {
        Some(renderer::Texture::Constant(color))
    }
}
