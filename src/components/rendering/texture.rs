extern crate gfx_device_gl;
extern crate gfx;

use renderer;
use renderer::target::ColorFormat;
pub use self::gfx::tex::Kind;
use self::gfx::Factory;
use self::gfx::format::{Formatted, SurfaceTyped};
use asset_manager::{AssetLoader, Assets};
use gfx_device::GfxLoader;

#[derive(Clone)]
/// Variants of this enum hold `amethyst_renderer::Texture`.
pub enum TextureInner {
    OpenGL {
        texture: renderer::Texture<gfx_device_gl::Resources>,
    },
    #[cfg(windows)]
    Direct3D {
        // stub
    },
    Null,
}

#[derive(Clone)]
/// This struct represents an image. It is a part of a `Renderable`.
pub struct Texture {
    pub texture_inner: TextureInner,
}

/// A struct for creating a new texture from raw data
pub struct TextureLoadData<'a> {
    pub kind: Kind,
    pub raw: &'a [&'a [<<ColorFormat as Formatted>::Surface as SurfaceTyped>::DataType]],
}

impl<'a> AssetLoader<Texture> for TextureLoadData<'a> {
    /// # Panics
    /// Panics if factory isn't registered as loader.
    fn from_data(assets: &mut Assets, load_data: TextureLoadData) -> Option<Texture> {
        let factory_inner = assets.get_loader_mut::<GfxLoader>().expect("Unable to retrieve factory");
        let texture_inner = match *factory_inner {
            GfxLoader::OpenGL { ref mut factory } => {
                let shader_resource_view = match factory.create_texture_const::<ColorFormat>(load_data.kind, load_data.raw) {
                    Ok((_, shader_resource_view)) => shader_resource_view,
                    Err(_) => return None,
                };
                let texture = renderer::Texture::Texture(shader_resource_view);
                TextureInner::OpenGL { texture: texture }
            }
            #[cfg(windows)]
            GfxLoader::Direct3D {} => unimplemented!(),
            GfxLoader::Null => TextureInner::Null,
        };
        Some(Texture { texture_inner: texture_inner })
    }
}

impl AssetLoader<Texture> for [f32; 4] {
    fn from_data(_: &mut Assets, color: [f32; 4]) -> Option<Texture> {
        let texture = renderer::Texture::Constant(color);
        let texture_inner = TextureInner::OpenGL { texture: texture };
        Some(Texture { texture_inner: texture_inner })
    }
}
