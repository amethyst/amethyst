pub use imagefmt::Error as ImageError;

use std::io::Cursor;

use assets::{Result, ResultExt, SimpleFormat};

use gfx_hal::{Backend, Device};
use gfx_hal::format::Format;
use gfx_hal::memory::Pod;
use gfx_hal::image::{AaMode, Kind, SamplerInfo};

use imagefmt::{self, ColFmt, Image};

use epoch::CurrentEpoch;
use memory::Allocator;
use texture::{Texture, TextureBuilder};
use upload::Uploader;

/// Texture metadata, used while loading
#[derive(Debug, Clone)]
pub struct TextureMetadata {
    /// Sampler info
    pub sampler: Option<SamplerInfo>,
    /// Mipmapping
    pub mip_levels: Option<u8>,
    /// Texture size
    pub size: Option<(u16, u16)>,
    /// Surface type
    pub format: Option<Format>,
}

impl Default for TextureMetadata {
    fn default() -> Self {
        Self {
            sampler: None,
            mip_levels: None,
            size: None,
            format: None,
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
    pub fn with_format(mut self, format: Format) -> Self {
        self.format = Some(format);
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
            .chain_err(|| "Image decoding failed")
    }
}

impl<B> SimpleFormat<Texture<B>> for JpgFormat
where
    B: Backend,
{
    const NAME: &'static str = "JPEG";

    type Options = TextureMetadata;

    fn import(&self, bytes: Vec<u8>, options: TextureMetadata) -> Result<TextureData> {
        self.from_data(bytes, options)
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
            .chain_err(|| "Image decoding failed")
    }
}

impl<B> SimpleFormat<Texture<B>> for PngFormat
where
    B: Backend,
{
    const NAME: &'static str = "PNG";

    type Options = TextureMetadata;

    fn import(&self, bytes: Vec<u8>, options: TextureMetadata) -> Result<TextureData> {
        self.from_data(bytes, options)
    }
}

/// Allows loading of BMP files.
#[derive(Clone)]
pub struct BmpFormat;

impl<B> SimpleFormat<Texture<B>> for BmpFormat
where
    B: Backend,
{
    const NAME: &'static str = "BMP";

    type Options = TextureMetadata;

    fn import(&self, bytes: Vec<u8>, options: TextureMetadata) -> Result<TextureData> {
        // TODO: consider reading directly into GPU-visible memory
        // TODO: as noted by @omni-viral.
        imagefmt::bmp::read(&mut Cursor::new(bytes), ColFmt::RGBA)
            .map(|raw| TextureData::Image(ImageData { raw }, options))
            .chain_err(|| "Image decoding failed")
    }
}

/// Create a texture asset.
pub fn create_texture_asset<B>(
    data: TextureData,
    allocator: &mut Allocator<B>,
    uploader: &mut Uploader<B>,
    current: &CurrentEpoch,
    device: &B::Device,
) -> Result<Texture<B>>
where
    B: Backend,
{
    use self::TextureData::*;
    match data {
        Image(image_data, options) => {
            create_texture_asset_from_image(image_data, options, allocator, uploader, current, device)
        }

        Rgba(color, options) => {
            let tb = apply_options(Texture::<B>::from_color_val(color), options);
            tb.build(allocator, uploader, current, device)
                .chain_err(|| "Failed to build texture")
        }

        F32(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            tb.build(allocator, uploader, current, device)
                .chain_err(|| "Failed to build texture")
        }

        F64(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            tb.build(allocator, uploader, current, device)
                .chain_err(|| "Failed to build texture")
        }

        U8(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            tb.build(allocator, uploader, current, device)
                .chain_err(|| "Failed to build texture")
        }

        U16(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            tb.build(allocator, uploader, current, device)
                .chain_err(|| "Failed to build texture")
        }

        U32(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            tb.build(allocator, uploader, current, device)
                .chain_err(|| "Failed to build texture")
        }

        U64(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            tb.build(allocator, uploader, current, device)
                .chain_err(|| "Failed to build texture")
        }
    }
}

fn apply_options(
    mut tb: TextureBuilder,
    metadata: TextureMetadata,
) -> TextureBuilder {
    match metadata.sampler {
        Some(sampler) => tb = tb.with_sampler(sampler),
        _ => (),
    }
    match metadata.mip_levels {
        Some(mip) => tb = tb.mip_levels(mip),
        _ => (),
    }
    match metadata.size {
        Some((w, h)) => tb = tb.with_kind(Kind::D2(w, h, AaMode::Single)),
        _ => (),
    }
    match metadata.format {
        Some(format) => tb = tb.with_format(format),
        _ => (),
    }

    tb
}

fn create_texture_asset_from_image<B>(
    image: ImageData,
    options: TextureMetadata,
    allocator: &mut Allocator<B>,
    uploader: &mut Uploader<B>,
    current: &CurrentEpoch,
    device: &B::Device,
) -> Result<Texture<B>>
where
    B: Backend,
{
    fn convert_color_format(fmt: ColFmt) -> Option<Format> {
        match fmt {
            ColFmt::Auto => unreachable!(),
            ColFmt::RGBA => Some(Format::Rgba8Unorm),
            ColFmt::BGRA => Some(Format::Bgra8Unorm),
            _ => None,
        }
    }

    let image = image.raw;
    let fmt = convert_color_format(image.fmt)
        .chain_err(|| format!("Unsupported color format {:?}", image.fmt))?;

    if image.w > u16::max_value() as usize || image.h > u16::max_value() as usize {
        bail!(
            "Unsupported texture size (expected: ({}, {}), got: ({}, {})",
            u16::max_value(),
            u16::max_value(),
            image.w,
            image.h
        );
    }

    apply_options(
        TextureBuilder::new(image.buf)
            .with_format(fmt)
            .with_kind(Kind::D2(image.w as u16, image.h as u16, AaMode::Single)),
        options,
    )
    .build(allocator, uploader, current, device)
    .chain_err(|| "Failed to create texture from image")
}
