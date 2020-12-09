use amethyst_assets::Handle;
use amethyst_rendy::{
    mtl::{Material, TextureOffset},
    sprite::Sprite,
    types::Texture,
};
use minterpolate::InterpolationPrimitive;
use serde::{Deserialize, Serialize};

use crate::{AnimationSampling, ApplyData, BlendMethod};

/// Sampler primitive for Material animations
/// Note that material can only ever be animated with `Step`, or a panic will occur.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MaterialPrimitive {
    /// Dynamically altering the texture rendered
    #[serde(skip)]
    Texture(Handle<Texture>),
    /// Dynamically altering the section of the texture rendered.
    Offset((f32, f32), (f32, f32)),
}

impl InterpolationPrimitive for MaterialPrimitive {
    fn add(&self, _: &Self) -> Self {
        panic!("Cannot add MaterialPrimitive")
    }

    fn sub(&self, _: &Self) -> Self {
        panic!("Cannot sub MaterialPrimitive")
    }

    fn mul(&self, _: f32) -> Self {
        panic!("Cannot mul MaterialPrimitive")
    }

    fn dot(&self, _: &Self) -> f32 {
        panic!("Cannot dot MaterialPrimitive")
    }

    fn magnitude2(&self) -> f32 {
        panic!("Cannot magnitude2 MaterialPrimitive")
    }

    fn magnitude(&self) -> f32 {
        panic!("Cannot magnitude MaterialPrimitive")
    }

    fn normalize(&self) -> Self {
        panic!("Cannot normalize MaterialPrimitive")
    }
}

impl From<Sprite> for MaterialPrimitive {
    fn from(sprite: Sprite) -> Self {
        let tex_coords = &sprite.tex_coords;
        MaterialPrimitive::Offset(
            (tex_coords.left, tex_coords.right),
            (tex_coords.top, tex_coords.bottom),
        )
    }
}

impl<'a> From<&'a Sprite> for MaterialPrimitive {
    fn from(sprite: &'a Sprite) -> Self {
        let tex_coords = &sprite.tex_coords;
        MaterialPrimitive::Offset(
            (tex_coords.left, tex_coords.right),
            (tex_coords.top, tex_coords.bottom),
        )
    }
}

/// Channels that are animatable on `Material`
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MaterialChannel {
    /// Animating the texture used for the albedo
    AlbedoTexture,
    /// Animating the texture used for the emission.
    EmissionTexture,
    /// Animating the texture used for the normal
    NormalTexture,
    /// Animating the texture used for the metallic and roughness
    MetallicRoughnessTexture,
    /// Animating the texture used for the ambient occlusion
    AmbientOcclusionTexture,
    /// Animating the texture used for the cavity
    CavityTexture,
    /// Animating the "window" used for all texture maps.
    UvOffset,
}

impl<'a> ApplyData<'a> for Material {
    type ApplyData = ();
}

fn offset(offset: &TextureOffset) -> MaterialPrimitive {
    MaterialPrimitive::Offset(offset.u, offset.v)
}

fn texture_offset(u: (f32, f32), v: (f32, f32)) -> TextureOffset {
    TextureOffset { u, v }
}

impl AnimationSampling for Material {
    type Primitive = MaterialPrimitive;
    type Channel = MaterialChannel;

    fn apply_sample(&mut self, channel: &Self::Channel, data: &Self::Primitive, _: &()) {
        match (channel, data) {
            (MaterialChannel::AlbedoTexture, MaterialPrimitive::Texture(i)) => {
                self.albedo = i.clone();
            }
            (MaterialChannel::EmissionTexture, MaterialPrimitive::Texture(i)) => {
                self.emission = i.clone();
            }
            (MaterialChannel::NormalTexture, MaterialPrimitive::Texture(i)) => {
                self.normal = i.clone();
            }
            (MaterialChannel::MetallicRoughnessTexture, MaterialPrimitive::Texture(i)) => {
                self.metallic_roughness = i.clone();
            }
            (MaterialChannel::AmbientOcclusionTexture, MaterialPrimitive::Texture(i)) => {
                self.ambient_occlusion = i.clone();
            }
            (MaterialChannel::CavityTexture, MaterialPrimitive::Texture(i)) => {
                self.cavity = i.clone();
            }

            (MaterialChannel::UvOffset, MaterialPrimitive::Offset(u, v)) => {
                self.uv_offset = texture_offset(*u, *v)
            }

            _ => panic!("Bad combination of data in Material animation"),
        }
    }

    fn current_sample(&self, channel: &Self::Channel, _: &()) -> Self::Primitive {
        match *channel {
            MaterialChannel::AlbedoTexture => MaterialPrimitive::Texture(self.albedo.clone()),
            MaterialChannel::EmissionTexture => MaterialPrimitive::Texture(self.emission.clone()),
            MaterialChannel::NormalTexture => MaterialPrimitive::Texture(self.normal.clone()),
            MaterialChannel::MetallicRoughnessTexture => {
                MaterialPrimitive::Texture(self.metallic_roughness.clone())
            }
            MaterialChannel::AmbientOcclusionTexture => {
                MaterialPrimitive::Texture(self.ambient_occlusion.clone())
            }
            MaterialChannel::CavityTexture => MaterialPrimitive::Texture(self.cavity.clone()),
            MaterialChannel::UvOffset => offset(&self.uv_offset),
        }
    }

    fn default_primitive(_: &Self::Channel) -> Self::Primitive {
        panic!("Blending is not applicable to Material animation")
    }

    fn blend_method(&self, _: &Self::Channel) -> Option<BlendMethod> {
        None
    }
}
