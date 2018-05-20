pub use imagefmt::Error as ImageError;

use std::io::Cursor;
use std::result::Result as StdResult;

use Renderer;
use amethyst_assets::SimpleFormat;
use failure::{Error, Fail};
use gfx::format::{ChannelType, SurfaceType};
use gfx::texture::SamplerInfo;
use gfx::traits::Pod;
use imagefmt;
use imagefmt::{ColFmt, Image};
use tex::{Texture, TextureBuilder};
use {ErrorKind, Result};

/// Texture metadata, used while loading
#[derive(Debug, Clone)]
pub struct TextureMetadata {
    /// Sampler info
    pub sampler: Option<SamplerInfo>,
    /// Mipmapping
    pub mip_levels: Option<u8>,
    /// Texture size
    pub size: Option<(u16, u16)>,
    /// Dynamic texture
    pub dynamic: bool,
    /// Surface type
    pub format: Option<SurfaceType>,
    /// Channel type
    pub channel: Option<ChannelType>,
}

impl Default for TextureMetadata {
    fn default() -> Self {
        Self {
            sampler: None,
            mip_levels: None,
            size: None,
            dynamic: false,
            format: None,
            channel: None,
        }
    }
}

impl TextureMetadata {
    /// Sampler info
    pub fn with_sampler(mut self, info: SamplerInfo) -> Self {
        self.sampler = Some(info);
        self
    }

    /// Mipmapping
    pub fn with_mip_levels(mut self, mip_levels: u8) -> Self {
        self.mip_levels = Some(mip_levels);
        self
    }

    /// Texture size
    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.size = Some((width, height));
        self
    }

    /// Surface type
    pub fn with_format(mut self, format: SurfaceType) -> Self {
        self.format = Some(format);
        self
    }

    /// Channel type
    pub fn with_channel(mut self, channel: ChannelType) -> Self {
        self.channel = Some(channel);
        self
    }

    /// Texture is dynamic
    pub fn dynamic(mut self, d: bool) -> Self {
        self.dynamic = d;
        self
    }
}

/// Texture data for loading
#[derive(Debug, Clone)]
pub enum TextureData {
    /// Image data
    Image(ImageData, TextureMetadata),

    /// Color
    Rgba([f32; 4], TextureMetadata),

    /// Float data
    F32(Vec<f32>, TextureMetadata),

    /// Float data
    F64(Vec<f64>, TextureMetadata),

    /// Byte data
    U8(Vec<u8>, TextureMetadata),

    /// Byte data
    U16(Vec<u16>, TextureMetadata),

    /// Byte data
    U32(Vec<u32>, TextureMetadata),

    /// Byte data
    U64(Vec<u64>, TextureMetadata),
}

impl From<[f32; 4]> for TextureData {
    fn from(color: [f32; 4]) -> Self {
        TextureData::Rgba(color, Default::default())
    }
}

impl From<[f32; 3]> for TextureData {
    fn from(color: [f32; 3]) -> Self {
        [color[0], color[1], color[2], 1.0].into()
    }
}

impl TextureData {
    /// Creates texture data from color.
    pub fn color(value: [f32; 4]) -> Self {
        TextureData::Rgba(value, Default::default())
    }
}

/// ImageData provided by formats, can be interpreted as a texture.
#[derive(Clone, Debug)]
pub struct ImageData {
    /// The raw image data.
    pub raw: Image<u8>,
}
/// Allows loading of jpg or jpeg files.
#[derive(Clone)]
pub struct JpgFormat;

impl JpgFormat {
    /// Load Jpg from memory buffer
    pub fn from_data(&self, data: Vec<u8>, options: TextureMetadata) -> Result<TextureData> {
        imagefmt::jpeg::read(&mut Cursor::new(data), ColFmt::RGBA)
            .map(|raw| TextureData::Image(ImageData { raw }, options))
            .map_err(|e| e.context(ErrorKind::DecodeImage).into())
    }
}

impl SimpleFormat<Texture> for JpgFormat {
    const NAME: &'static str = "JPEG";

    type Options = TextureMetadata;

    fn import(&self, bytes: Vec<u8>, options: TextureMetadata) -> StdResult<TextureData, Error> {
        self.from_data(bytes, options).map_err(|e| e.into())
    }
}

/// Allows loading of PNG files.
#[derive(Clone)]
pub struct PngFormat;

impl PngFormat {
    /// Load Png from memory buffer
    pub fn from_data(&self, data: Vec<u8>, options: TextureMetadata) -> Result<TextureData> {
        imagefmt::png::read(&mut Cursor::new(data), ColFmt::RGBA)
            .map(|raw| TextureData::Image(ImageData { raw }, options))
            .map_err(|e| e.context(ErrorKind::TextureCreation).into())
    }
}

impl SimpleFormat<Texture> for PngFormat {
    const NAME: &'static str = "PNG";

    type Options = TextureMetadata;

    fn import(&self, bytes: Vec<u8>, options: TextureMetadata) -> StdResult<TextureData, Error> {
        self.from_data(bytes, options).map_err(|e| e.into())
    }
}

/// Allows loading of BMP files.
#[derive(Clone)]
pub struct BmpFormat;

impl SimpleFormat<Texture> for BmpFormat {
    const NAME: &'static str = "BMP";

    type Options = TextureMetadata;

    fn import(&self, bytes: Vec<u8>, options: TextureMetadata) -> StdResult<TextureData, Error> {
        // TODO: consider reading directly into GPU-visible memory
        // TODO: as noted by @omni-viral.
        imagefmt::bmp::read(&mut Cursor::new(bytes), ColFmt::RGBA)
            .map(|raw| TextureData::Image(ImageData { raw }, options))
            .map_err(|e| e.context(ErrorKind::TextureCreation).into())
    }
}

/// Create a texture asset.
pub fn create_texture_asset(data: TextureData, renderer: &mut Renderer) -> Result<Texture> {
    use self::TextureData::*;
    match data {
        Image(image_data, options) => {
            create_texture_asset_from_image(image_data, options, renderer)
        }

        Rgba(color, options) => {
            let tb = apply_options(Texture::from_color_val(color), options);
            renderer.create_texture(tb)
        }

        F32(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb)
        }

        F64(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb)
        }

        U8(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb)
        }

        U16(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb)
        }

        U32(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb)
        }

        U64(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb)
        }
    }.map_err(|e| e.context(ErrorKind::TextureCreation).into())
}

fn apply_options<D, T>(
    mut tb: TextureBuilder<D, T>,
    metadata: TextureMetadata,
) -> TextureBuilder<D, T>
where
    D: AsRef<[T]>,
    T: Pod + Copy,
{
    match metadata.sampler {
        Some(sampler) => tb = tb.with_sampler(sampler),
        _ => (),
    }
    match metadata.mip_levels {
        Some(mip) => tb = tb.mip_levels(mip),
        _ => (),
    }
    match metadata.size {
        Some((w, h)) => tb = tb.with_size(w, h),
        _ => (),
    }
    if metadata.dynamic {
        tb = tb.dynamic(true);
    }
    match metadata.format {
        Some(format) => tb = tb.with_format(format),
        _ => (),
    }
    match metadata.channel {
        Some(channel) => tb = tb.with_channel_type(channel),
        _ => (),
    }

    tb
}

fn create_texture_asset_from_image(
    image: ImageData,
    options: TextureMetadata,
    renderer: &mut Renderer,
) -> Result<Texture> {
    fn convert_color_format(fmt: ColFmt) -> Result<SurfaceType> {
        match fmt {
            ColFmt::Auto => unreachable!(),
            ColFmt::RGBA => Ok(SurfaceType::R8_G8_B8_A8),
            ColFmt::BGRA => Ok(SurfaceType::B8_G8_R8_A8),
            _ => Err(format_err!("Unsupported color format {:?}", fmt)
                .context(ErrorKind::TextureCreation)
                .into()),
        }
    }

    let image = image.raw;
    let fmt = convert_color_format(image.fmt)?;

    if image.w > u16::max_value() as usize || image.h > u16::max_value() as usize {
        return Err(format_err!(
            "Unsupported texture size (max allowed: ({}, {}), got: ({}, {})",
            u16::max_value(),
            u16::max_value(),
            image.w,
            image.h
        ).context(ErrorKind::TextureCreation)
            .into());
    }

    let tb = apply_options(
        TextureBuilder::new(image.buf)
            .with_format(fmt)
            .with_size(image.w as u16, image.h as u16),
        options,
    );

    renderer
        .create_texture(tb)
        .map_err(|e| e.context(ErrorKind::TextureCreation).into())
}

#[cfg(test)]
mod tests {
    use super::TextureData;

    #[test]
    fn texture_data_from_f32_3() {
        match TextureData::from([0.25, 0.50, 0.75]) {
            TextureData::Rgba(color, _) => {
                assert_eq!(color, [0.25, 0.50, 0.75, 1.0]);
            }
            _ => panic!("Expected [f32; 3] to turn into TextureData::Rgba"),
        }
    }
}
