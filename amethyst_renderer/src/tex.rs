//! Texture resource.

pub use gfx::texture::{FilterMethod, WrapMode};

use error::Result;
use gfx::texture::{Info, SamplerInfo};
use gfx::traits::Pod;
use std::marker::PhantomData;
use types::{Factory, RawShaderResourceView, RawTexture, Sampler};

/// Handle to a GPU texture resource.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Texture {
    sampler: Sampler,
    texture: RawTexture,
    view: RawShaderResourceView,
}

impl Texture {
    /// Builds a new texture with the given raw texture data.
    pub fn from_data<T: Pod, D: AsRef<[T]>>(data: D) -> TextureBuilder<D, T> {
        TextureBuilder::new(data)
    }

    /// Builds a new texture with the given raw texture data.
    pub fn from_color_val<C: Into<[f32; 4]>>(rgba: C) -> TextureBuilder<[u8; 4], u8> {
        TextureBuilder::from_color_val(rgba)
    }

    /// Returns the sampler for the texture.
    pub fn sampler(&self) -> &Sampler {
        &self.sampler
    }

    /// Returns the texture's raw shader resource view.
    pub fn view(&self) -> &RawShaderResourceView {
        &self.view
    }
}

/// Builds new textures.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TextureBuilder<D, T> {
    data: D,
    info: Info,
    sampler: SamplerInfo,
    pd: PhantomData<T>,
}

impl TextureBuilder<[u8; 4], u8> {
    /// Creates a new `TextureBuilder` from the given RGBA color value.
    pub fn from_color_val<C: Into<[f32; 4]>>(rgba: C) -> Self {
        let color = rgba.into();
        let data: [u8; 4] = [(color[0] * 255.0) as u8,
                                   (color[1] * 255.0) as u8,
                                   (color[2] * 255.0) as u8,
                                   (color[3] * 255.0) as u8];

        TextureBuilder::new(data)
    }
}

impl<D, T> TextureBuilder<D, T>
    where D: AsRef<[T]>,
          T: Pod,
{
    /// Creates a new `TextureBuilder` with the given raw texture data.
    pub fn new(data: D) -> Self {
        use gfx::SHADER_RESOURCE;
        use gfx::format::SurfaceType;
        use gfx::memory::Usage;
        use gfx::texture::{AaMode, Kind};

        TextureBuilder {
            data: data,
            info: Info {
                kind: Kind::D2(1, 1, AaMode::Single),
                levels: 1,
                format: SurfaceType::R8_G8_B8_A8,
                bind: SHADER_RESOURCE,
                usage: Usage::Dynamic,
            },
            sampler: SamplerInfo::new(FilterMethod::Scale, WrapMode::Clamp),
            pd: PhantomData,
        }
    }

    /// Sets the number of mipmap levels to generate.
    ///
    /// FIXME: Only encoders can generate mipmap levels.
    pub fn mip_levels(mut self, val: u8) -> Self {
        self.info.levels = val;
        self
    }

    /// Sets the texture length and width in pixels.
    pub fn with_size(mut self, l: usize, w: usize) -> Self {
        use gfx::texture::{AaMode, Kind};
        self.info.kind = Kind::D2(l as u16, w as u16, AaMode::Single);
        self
    }

    /// Sets whether the texture is mutable or not.
    pub fn dynamic(mut self, mutable: bool) -> Self {
        use gfx::memory::Usage;
        self.info.usage = if mutable { Usage::Dynamic } else { Usage::Data };
        self
    }

    /// Builds and returns the new texture.
    pub(crate) fn build(self, fac: &mut Factory) -> Result<Texture> {
        use gfx::Factory;
        use gfx::format::{ChannelType, Swizzle};
        use gfx::memory::cast_slice;
        use gfx::texture::ResourceDesc;

        let chan = ChannelType::Srgb;
        let tex = fac.create_texture_raw(self.info, Some(chan), Some(&[cast_slice(self.data.as_ref())]))?;

        let desc = ResourceDesc {
            channel: ChannelType::Srgb,
            layer: None,
            min: 1,
            max: self.info.levels,
            swizzle: Swizzle::new(),
        };

        let view = fac.view_texture_as_shader_resource_raw(&tex, desc)?;
        let sampler = fac.create_sampler(self.sampler);

        Ok(Texture {
            sampler: sampler,
            texture: tex,
            view: view,
        })
    }
}
