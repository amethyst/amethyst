//! Module containing utility methods to easily create `Material` and `Texture` handles using a minimal amount of code.

use amethyst_assets::{AssetStorage, Handle, Loader, Progress};
use amethyst_core::specs::{Read, ReadExpect};
use amethyst_renderer::{Material, MaterialDefaults, PngFormat, Texture, TextureMetadata};

use shred_derive::SystemData;

/// SystemData used to create `Material` instances in various ways.
///
/// # Example
/// ```rust, ignore
/// let mat = world.exec(|creator: MaterialCreator|
///     creator.material_from_color([0.0, 1.0, 0.0, 1.0], ())
/// );
/// ```
#[derive(SystemData)]
pub struct MaterialCreator<'a> {
    loader: ReadExpect<'a, Loader>,
    storage: Read<'a, AssetStorage<Texture>>,
    defaults: ReadExpect<'a, MaterialDefaults>,
}

impl<'a> MaterialCreator<'a> {
    /// Generate a `Material` from a color.
    pub fn material_from_color<T: Progress>(
        &self,
        color: [f32; 4],
        progress_counter: T,
    ) -> Material {
        let albedo = self
            .loader
            .load_from_data(color.into(), progress_counter, &self.storage);
        self.material_from_texture(albedo)
    }

    /// Generate a `Material` from a filesystem path pointing to a png image.
    /// Uses the "resource path" provided at Amethyst's initialization.
    pub fn material_from_png<T: Progress, S: Into<String>>(
        &self,
        path: S,
        progress_counter: T,
    ) -> Material {
        self.material_from_texture(self.loader.load(
            path,
            PngFormat,
            TextureMetadata::srgb(),
            progress_counter,
            &self.storage,
        ))
    }

    /// Generate a `Material` from a texture handle.
    pub fn material_from_texture(&self, texture: Handle<Texture>) -> Material {
        Material {
            albedo: texture,
            ..self.defaults.0.clone()
        }
    }
}
