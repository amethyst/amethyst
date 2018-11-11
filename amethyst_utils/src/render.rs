//! Module containing utility methods to easily create `Material` and `Texture` handles using a minimal amount of code.

use amethyst_assets::{AssetStorage, Handle, Loader};
use amethyst_core::specs::World;
use amethyst_renderer::{Material, MaterialDefaults, PngFormat, Texture, TextureMetadata};

/// Generate a `Material` from a color.
pub fn material_from_color(
    color: [f32; 4],
    loader: &Loader,
    storage: &AssetStorage<Texture>,
    material_defaults: &MaterialDefaults,
) -> Material {
    let albedo = loader.load_from_data(color.into(), (), &storage);
    material_from_texture(albedo, material_defaults)
}

/// Generate a `Material` from a texture handle.
pub fn material_from_texture(texture: Handle<Texture>, defaults: &MaterialDefaults) -> Material {
    Material {
        albedo: texture,
        ..defaults.0.clone()
    }
}

/// Generate a `Material` from a path pointing to a png image.
pub fn material_from_png(
    path: String,
    loader: &Loader,
    storage: &AssetStorage<Texture>,
    material_defaults: &MaterialDefaults,
) -> Material {
    material_from_texture(
        loader.load(path, PngFormat, TextureMetadata::srgb(), (), &storage),
        material_defaults,
    )
}

/// Generate a `Material` from a color.
pub fn world_material_from_color(color: [f32; 4], world: &World) -> Material {
    material_from_color(
        color,
        &world.read_resource(),
        &world.read_resource(),
        &world.read_resource(),
    )
}

/// Generate a `Material` from a path pointing to a png image.
pub fn world_material_from_png(path: String, world: &World) -> Material {
    material_from_png(
        path,
        &world.read_resource(),
        &world.read_resource(),
        &world.read_resource(),
    )
}
