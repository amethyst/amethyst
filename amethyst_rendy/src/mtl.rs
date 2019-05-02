//! Physically-based material.

use crate::types::Texture;
use amethyst_assets::{Asset, Handle};
use amethyst_core::ecs::prelude::DenseVecStorage;
use rendy::hal::Backend;

/// Material reference this part of the texture
#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TextureOffset {
    /// Start and end offset for U coordinate
    pub u: (f32, f32),
    /// Start and end offset for V coordinate
    pub v: (f32, f32),
}

impl Default for TextureOffset {
    fn default() -> Self {
        TextureOffset {
            u: (0., 1.),
            v: (0., 1.),
        }
    }
}

/// A physically based Material with metallic workflow, fully utilized in PBR render pass.
#[derive(Debug, Clone, PartialEq)]
pub struct Material<B: Backend> {
    /// Alpha cutoff: the value at which we do not draw the pixel
    pub alpha_cutoff: f32,
    /// Diffuse map.
    pub albedo: Handle<Texture<B>>,
    /// Emission map.
    pub emission: Handle<Texture<B>>,
    /// Normal map.
    pub normal: Handle<Texture<B>>,
    /// Metallic-roughness map. (B channel metallic, G channel roughness)
    pub metallic_roughness: Handle<Texture<B>>,
    /// Ambient occlusion map.
    pub ambient_occlusion: Handle<Texture<B>>,
    /// Cavity map.
    pub cavity: Handle<Texture<B>>,
    /// Texture offset
    pub uv_offset: TextureOffset,
}

impl<B: Backend> Asset for Material<B> {
    const NAME: &'static str = "renderer::Material";
    type Data = Self;
    type HandleStorage = DenseVecStorage<Handle<Self>>;
}

/// A resource providing default textures for `Material`.
/// These will be be used by the renderer in case a texture
/// handle points to a texture which is not loaded already.
/// Additionally, you can use it to fill up the fields of
/// `Material` you don't want to specify.
#[derive(Clone)]
pub struct MaterialDefaults<B: Backend>(pub Material<B>);

pub trait StaticTextureSet<'a, B: Backend>:
    Clone + Copy + std::fmt::Debug + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static
{
    type Iter: Iterator<Item = &'a Handle<Texture<B>>>;
    fn textures(mat: &'a Material<B>) -> Self::Iter;
    fn len() -> usize {
        1
    }
}

pub type FullTextureSet = (
    TexAlbedo,
    TexEmission,
    TexNormal,
    TexMetallicRoughness,
    TexAmbientOcclusion,
    TexCavity,
);

macro_rules! impl_texture {
    ($name:ident, $prop:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name;
        impl<'a, B: Backend> StaticTextureSet<'a, B> for $name {
            type Iter = std::iter::Once<&'a Handle<Texture<B>>>;
            #[inline(always)]
            fn textures(mat: &'a Material<B>) -> Self::Iter {
                std::iter::once(&mat.$prop)
            }
        }
    };
}

impl_texture!(TexAlbedo, albedo);
impl_texture!(TexEmission, emission);
impl_texture!(TexNormal, normal);
impl_texture!(TexMetallicRoughness, metallic_roughness);
impl_texture!(TexAmbientOcclusion, ambient_occlusion);
impl_texture!(TexCavity, cavity);

macro_rules! recursive_iter {
    (@value $first:expr, $($rest:expr),*) => { $first.chain(recursive_iter!(@value $($rest),*)) };
    (@value $last:expr) => { $last };
    (@type $first:ty, $($rest:ty),*) => { std::iter::Chain<$first, recursive_iter!(@type $($rest),*)> };
    (@type $last:ty) => { $last };
}

macro_rules! impl_texture_set_tuple {
    ($($from:ident),*) => {
        impl<'a, BE: Backend, $($from,)*> StaticTextureSet<'a, BE> for ($($from),*,)
        where
            $($from: StaticTextureSet<'a, BE>),*,
        {
            type Iter = recursive_iter!(@type $($from::Iter),*);
            #[inline(always)]
            fn textures(mat: &'a Material<BE>) -> Self::Iter {
                recursive_iter!(@value $($from::textures(mat)),*)
            }
            fn len() -> usize {
                $($from::len() + )* 0
            }
        }
    }
}

impl_texture_set_tuple!(A);
impl_texture_set_tuple!(A, B);
impl_texture_set_tuple!(A, B, C);
impl_texture_set_tuple!(A, B, C, D);
impl_texture_set_tuple!(A, B, C, D, E);
impl_texture_set_tuple!(A, B, C, D, E, F);
