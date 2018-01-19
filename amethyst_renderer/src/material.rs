//! Physically-based material.

use assets::{AssetStorage, Loader};
use gfx_hal::Backend;
use specs::{Component, DenseVecStorage, World};
use texture::{Texture, TextureHandle};

/// Material struct.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Material<B: Backend> {
    /// Diffuse map.
    pub albedo: TextureHandle<B>,
    /// Emission map.
    pub emission: TextureHandle<B>,
    /// Normal map.
    pub normal: TextureHandle<B>,
    /// Metallic map.
    pub metallic: TextureHandle<B>,
    /// Roughness map.
    pub roughness: TextureHandle<B>,
    /// Ambient occlusion map.
    pub ambient_occlusion: TextureHandle<B>,
    /// Caveat map.
    pub caveat: TextureHandle<B>,
}

impl<B> Component for Material<B>
where
    B: Backend,
{
    type Storage = DenseVecStorage<Self>;
}

/// A resource providing default textures for `Material`.
/// These will be be used by the renderer in case a texture
/// handle points to a texture which is not loaded already.
/// Additionally, you can use it to fill up the fields of
/// `Material` you don't want to specify.
#[derive(Clone)]
pub struct MaterialDefaults<B: Backend>(pub Material<B>);


pub fn create_default_material<B>(world: &World) -> Material<B>
where
    B: Backend,
{
    let loader = world.read_resource::<Loader>();

    let albedo = [0.5, 0.5, 0.5, 1.0].into();
    let emission = [0.0; 4].into();
    let normal = [0.5, 0.5, 1.0, 1.0].into();
    let metallic = [0.0; 4].into();
    let roughness = [0.5; 4].into();
    let ambient_occlusion = [1.0; 4].into();
    let caveat = [1.0; 4].into();

    let tex_storage = world.read_resource::<AssetStorage<Texture<B>>>();

    let albedo = loader.load_from_data(albedo, (), &tex_storage);
    let emission = loader.load_from_data(emission, (), &tex_storage);
    let normal = loader.load_from_data(normal, (), &tex_storage);
    let metallic = loader.load_from_data(metallic, (), &tex_storage);
    let roughness = loader.load_from_data(roughness, (), &tex_storage);
    let ambient_occlusion = loader.load_from_data(ambient_occlusion, (), &tex_storage);
    let caveat = loader.load_from_data(caveat, (), &tex_storage);

    Material {
        albedo,
        emission,
        normal,
        metallic,
        roughness,
        ambient_occlusion,
        caveat,
    }
}
