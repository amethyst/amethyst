//! Texture resource.

use error::Result;
use gfx::texture::{Kind, Info};
use gfx::traits::Pod;
use types::{Factory, RawTexture, RawShaderResourceView};

/// Handle to a GPU texture resource.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Texture {
    data: Vec<u8>,
    kind: Kind,
    texture: RawTexture,
    view: RawShaderResourceView,
}

impl Texture {
    /// Builds a new texture with the given raw texture data.
    pub fn new<'d, T: 'd, D: 'd>(data: D) -> TextureBuilder
        where T: Copy + Pod,
              D: Into<&'d [T]>
    {
        TextureBuilder::new(data)
    }

    /// Builds a new texture with the given raw texture data.
    pub fn from_color_val<C: Into<[f32; 4]>>(rgba: C) -> TextureBuilder {
        TextureBuilder::from_color_val(rgba)
    }
}

/// Builds new textures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextureBuilder {
    data: Vec<u8>,
    info: Info,
}

impl TextureBuilder {
    /// Creates a new `TextureBuilder` with the given raw texture data.
    pub fn new<'d, T: 'd, D>(data: D) -> TextureBuilder
        where T: Copy + Pod,
              D: Into<&'d [T]>
    {
        use gfx::Bind;
        use gfx::format::SurfaceType;
        use gfx::memory::{cast_slice, Usage};
        use gfx::texture::{AaMode, Kind};

        TextureBuilder {
            data: cast_slice(data.into()).to_vec(),
            info: Info {
                kind: Kind::D2(1, 1, AaMode::Single),
                levels: 1,
                format: SurfaceType::R8_G8_B8_A8,
                bind: Bind::empty(),
                usage: Usage::Dynamic,
            },
        }
    }
    
    /// Creates a new `TextureBuilder` from the given RGBA color value.
    pub fn from_color_val<C: Into<[f32; 4]>>(rgba: C) -> Self {
        let color = rgba.into();
        let data: [[u8; 4]; 1] = [[(color[0] * 255.0) as u8,
                                    (color[1] * 255.0) as u8,
                                    (color[2] * 255.0) as u8,
                                    (color[3] * 255.0) as u8]];

        TextureBuilder::new::<[u8; 4], &[[u8; 4]]>(&data)
    }

    /// Sets the number of mipmap levels to generate.
    pub fn with_mip_levels(mut self, val: u8) -> Self {
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
    pub fn is_mutable(mut self, mutable: bool) -> Self {
        use gfx::memory::Usage;

        self.info.usage = if mutable {
            Usage::Dynamic
        } else {
            Usage::Data
        };

        self
    }

    /// Builds and returns the new texture.
    pub fn build(self, fac: &mut Factory) -> Result<Texture> {
        use gfx::Factory;
        use gfx::format::{ChannelType, Swizzle};
        use gfx::texture::ResourceDesc;

        let chan = ChannelType::Srgb;
        let tex = fac.create_texture_raw(self.info, Some(chan), None)?;

        let desc = ResourceDesc {
            channel: ChannelType::Srgb,
            layer: None,
            min: 1,
            max: self.info.levels,
            swizzle: Swizzle::new(),
        };

        let view = fac.view_texture_as_shader_resource_raw(&tex, desc)?;

        Ok(Texture {
            data: self.data.to_owned(),
            kind: self.info.kind,
            texture: tex,
            view: view,
        })
    }
}
