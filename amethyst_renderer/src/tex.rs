//! Texture resource.

pub use gfx::texture::{FilterMethod, WrapMode};

use specs::{Component, DenseVecStorage};

use std::marker::PhantomData;

use error::Result;
use gfx::format::{ChannelType, SurfaceType};
use gfx::texture::{Info, SamplerInfo};
use gfx::traits::Pod;

use types::{ChannelFormat, Factory, RawShaderResourceView, RawTexture, Sampler, SurfaceFormat};

/// Handle to a GPU texture resource.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Texture {
    sampler: Sampler,
    texture: RawTexture,
    view: RawShaderResourceView,
}

impl Texture {
    /// Builds a new texture with the given raw texture data.
    pub fn from_data<T: Pod + Copy, D: AsRef<[T]>>(data: D) -> TextureBuilder<D, T> {
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
    channel_type: ChannelType,
    sampler: SamplerInfo,
    pd: PhantomData<T>,
}

impl TextureBuilder<[u8; 4], u8> {
    /// Creates a new `TextureBuilder` from the given RGBA color value.
    pub fn from_color_val<C: Into<[f32; 4]>>(rgba: C) -> Self {
        let color = rgba.into();
        let data: [u8; 4] = [
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
            (color[3] * 255.0) as u8,
        ];

        TextureBuilder::new(data)
    }
}

impl<D, T> TextureBuilder<D, T>
where
    D: AsRef<[T]>,
    T: Pod + Copy,
{
    /// Creates a new `TextureBuilder` with the given raw texture data.
    pub fn new(data: D) -> Self {
        use gfx::SHADER_RESOURCE;
        use gfx::memory::Usage;
        use gfx::texture::{AaMode, Kind};
        use gfx::format::{ChannelTyped, SurfaceTyped};

        TextureBuilder {
            data: data,
            info: Info {
                kind: Kind::D2(1, 1, AaMode::Single),
                levels: 1,
                format: SurfaceFormat::get_surface_type(),
                bind: SHADER_RESOURCE,
                usage: Usage::Dynamic,
            },
            channel_type: ChannelFormat::get_channel_type(),
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

    /// Sets the texture width and height in pixels.
    pub fn with_size(mut self, w: u16, h: u16) -> Self {
        use gfx::texture::{AaMode, Kind};
        self.info.kind = Kind::D2(w, h, AaMode::Single);
        self
    }

    /// Sets whether the texture is mutable or not.
    pub fn dynamic(mut self, mutable: bool) -> Self {
        use gfx::memory::Usage;
        self.info.usage = if mutable { Usage::Dynamic } else { Usage::Data };
        self
    }

    /// Sets the texture format
    pub fn with_format(mut self, format: SurfaceType) -> Self {
        self.info.format = format;
        self
    }

    /// Sets the texture channel type
    pub fn with_channel_type(mut self, channel_type: ChannelType) -> Self {
        self.channel_type = channel_type;
        self
    }

    /// Builds and returns the new texture.
    pub fn build(self, fac: &mut Factory) -> Result<Texture> {
        use gfx::Factory;
        use gfx::format::Swizzle;
        use gfx::memory::cast_slice;
        use gfx::texture::ResourceDesc;


        // This variable has to live here to make sure the flipped
        // buffer lives long enough. (If one exists)
        let mut v_flip_buffer;
        let mut data = self.data.as_ref();

        if cfg!(feature = "opengl") {
            let pixel_width = (self.info.format.get_total_bits() / 8) as usize;
            v_flip_buffer = Vec::with_capacity(data.len());
            let (w, h, _, _) = self.info.kind.get_dimensions();
            let w = w as usize;
            let h = h as usize;
            for y in 0..h {
                for x in 0..(w * pixel_width) {
                    v_flip_buffer.push(data[x + (h - y - 1) * w * pixel_width]);
                }
            }
            data = &v_flip_buffer;
        }

        let tex = fac.create_texture_raw(
            self.info,
            Some(self.channel_type),
            Some(&[cast_slice(data)]),
        )?;

        let desc = ResourceDesc {
            channel: self.channel_type,
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

impl Component for Texture {
    type Storage = DenseVecStorage<Self>;
}
