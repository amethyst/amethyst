//! Physically-based material.

use amethyst_assets::{
    erased_serde::private::serde::{de, de::SeqAccess, ser::SerializeSeq},
    prefab::{
        register_component_type,
        serde_diff::{ApplyContext, DiffContext},
        SerdeDiff,
    },
    Asset, Handle,
};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

use crate::types::Texture;

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TypeUuid)]
#[uuid = "e238c036-42e9-4d0e-9aa9-c6511c906820"]
pub struct Material {
    /// Alpha cutoff: the value at which we do not draw the pixel
    pub alpha_cutoff: f32,
    /// Diffuse map.
    pub albedo: Handle<Texture>,
    /// Emission map.
    pub emission: Handle<Texture>,
    /// Normal map.
    pub normal: Handle<Texture>,
    /// Metallic-roughness map. (B channel metallic, G channel roughness)
    pub metallic_roughness: Handle<Texture>,
    /// Ambient occlusion map.
    pub ambient_occlusion: Handle<Texture>,
    /// Cavity map.
    pub cavity: Handle<Texture>,
    /// Texture offset
    pub uv_offset: TextureOffset,
}

impl Asset for Material {
    fn name() -> &'static str {
        "renderer::Material"
    }
    type Data = Self;
}

impl Default for Material {
    fn default() -> Self {
        unimplemented!()
    }
}

impl SerdeDiff for Material {
    fn diff<'a, S: SerializeSeq>(
        &self,
        ctx: &mut DiffContext<'a, S>,
        other: &Self,
    ) -> Result<bool, <S as SerializeSeq>::Error> {
        unimplemented!()
    }

    fn apply<'de, A>(
        &mut self,
        seq: &mut A,
        ctx: &mut ApplyContext,
    ) -> Result<bool, <A as SeqAccess<'de>>::Error>
    where
        A: de::SeqAccess<'de>,
    {
        unimplemented!()
    }
}

register_component_type!(Material);

// impl From<Material> for Material {
//     fn from(material: Material) -> Self {
//         material
//     }
// }

/// A resource providing default textures for `Material`.
/// These will be be used by the renderer in case a texture
/// handle points to a texture which is not loaded already.
/// Additionally, you can use it to fill up the fields of
/// `Material` you don't want to specify.
#[derive(Debug, Clone)]
pub struct MaterialDefaults(pub Material);

/// Trait providing generic access to a collection of texture handles
pub trait StaticTextureSet<'a>:
    Clone + Copy + std::fmt::Debug + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static
{
    /// Iterator type to access this texture sets handles
    type Iter: Iterator<Item = &'a Handle<Texture>>;

    /// Returns an iterator to the textures associated with a given material.
    fn textures(mat: &'a Material) -> Self::Iter;

    /// ALWAYS RETURNS 1
    fn len() -> usize {
        1
    }
}

/// Type alias for a tuple collection of a complete PBR texture set.
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
        #[doc = "Macro Generated Texture Type"]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name;
        impl<'a> StaticTextureSet<'a> for $name {
            type Iter = std::iter::Once<&'a Handle<Texture>>;
            #[inline(always)]
            fn textures(mat: &'a Material) -> Self::Iter {
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
        impl<'a, $($from,)*> StaticTextureSet<'a> for ($($from),*,)
        where
            $($from: StaticTextureSet<'a>),*,
        {
            type Iter = recursive_iter!(@type $($from::Iter),*);
            #[inline(always)]
            fn textures(mat: &'a Material) -> Self::Iter {
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
