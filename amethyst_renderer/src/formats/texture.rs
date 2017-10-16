pub use imagefmt::Error as ImageError;

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::Cursor;
use std::sync::Arc;

use amethyst_assets::{BoxedErr, Format, Source};
use imagefmt;
use imagefmt::{ColFmt, Image};
use gfx::format::{ChannelType, SurfaceType};
use gfx::texture::SamplerInfo;
use gfx::traits::Pod;
use tex::{Texture, TextureBuilder};
use Renderer;

/// Texture metadata, used while loading
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
pub struct JpgFormat;

impl Format<Texture> for JpgFormat {
    const NAME: &'static str = "JPEG";

    type Options = TextureMetadata;

    fn import(
        &self,
        name: String,
        source: Arc<Source>,
        options: TextureMetadata,
    ) -> Result<TextureData, BoxedErr> {
        imagefmt::jpeg::read(&mut Cursor::new(source.load(&name)?), ColFmt::RGBA)
            .map(|raw| TextureData::Image(ImageData { raw }, options))
            .map_err(BoxedErr::new)
    }
}

/// Allows loading of PNG files.
pub struct PngFormat;

impl Format<Texture> for PngFormat {
    const NAME: &'static str = "JPEG";

    type Options = TextureMetadata;

    fn import(
        &self,
        name: String,
        source: Arc<Source>,
        options: TextureMetadata,
    ) -> Result<TextureData, BoxedErr> {
        imagefmt::png::read(&mut Cursor::new(source.load(&name)?), ColFmt::RGBA)
            .map(|raw| TextureData::Image(ImageData { raw }, options))
            .map_err(BoxedErr::new)
    }
}

/// Allows loading of BMP files.
pub struct BmpFormat;

impl Format<Texture> for BmpFormat {
    const NAME: &'static str = "BMP";

    type Options = TextureMetadata;

    fn import(
        &self,
        name: String,
        source: Arc<Source>,
        options: TextureMetadata,
    ) -> Result<TextureData, BoxedErr> {
        imagefmt::bmp::read(&mut Cursor::new(source.load(&name)?), ColFmt::RGBA)
            .map(|raw| TextureData::Image(ImageData { raw }, options))
            .map_err(BoxedErr::new)
    }
}

/// Error that can occur during texture creation
#[derive(Debug)]
pub enum TextureError {
    /// Error occured in renderer
    Renderer(::error::Error),

    /// Color format unsupported
    UnsupportedColorFormat(ColFmt),

    /// Texture is oversized
    UnsupportedSize {
        /// Maximum size of texture (width, height)
        max: (usize, usize),

        /// Image size (width, height)
        got: (usize, usize),
    },
}

impl Error for TextureError {
    fn description(&self) -> &str {
        match *self {
            TextureError::Renderer(ref err) => err.description(),
            TextureError::UnsupportedColorFormat(_) => "Unsupported color format",
            TextureError::UnsupportedSize { .. } => "Unsupported size",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            TextureError::Renderer(ref err) => Some(err),
            _ => None,
        }
    }
}

impl Display for TextureError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match *self {
            TextureError::Renderer(ref err) => write!(fmt, "Render error: {}", err),
            TextureError::UnsupportedColorFormat(col_fmt) => {
                write!(fmt, "Unsupported color format: {:?}", col_fmt)
            }
            TextureError::UnsupportedSize { max, got } => {
                write!(fmt, "Unsupported size. max: {:?}, got: {:?}", max, got)
            }
        }
    }
}

/// Create a texture asset.
pub fn create_texture_asset(
    data: TextureData,
    renderer: &mut Renderer,
) -> Result<Texture, BoxedErr> {
    use self::TextureData::*;
    match data {
        Image(image_data, options) => {
            create_texture_asset_from_image(image_data, options, renderer)
        }

        Rgba(color, options) => {
            let tb = apply_options(Texture::from_color_val(color), options);
            renderer.create_texture(tb).map_err(BoxedErr::new)
        }

        F32(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb).map_err(BoxedErr::new)
        }

        F64(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb).map_err(BoxedErr::new)
        }

        U8(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb).map_err(BoxedErr::new)
        }

        U16(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb).map_err(BoxedErr::new)
        }

        U32(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb).map_err(BoxedErr::new)
        }

        U64(data, options) => {
            let tb = apply_options(TextureBuilder::new(data), options);
            renderer.create_texture(tb).map_err(BoxedErr::new)
        }
    }
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
) -> Result<Texture, BoxedErr> {
    fn convert_color_format(fmt: ColFmt) -> Option<SurfaceType> {
        match fmt {
            ColFmt::Auto => unreachable!(),
            ColFmt::RGBA => Some(SurfaceType::R8_G8_B8_A8),
            ColFmt::BGRA => Some(SurfaceType::B8_G8_R8_A8),
            _ => None,
        }
    }

    let image = image.raw;
    let fmt = match convert_color_format(image.fmt) {
        Some(fmt) => fmt,
        None => {
            return Err(BoxedErr::new(
                TextureError::UnsupportedColorFormat(image.fmt),
            ))
        }
    };

    if image.w > u16::max_value() as usize || image.h > u16::max_value() as usize {
        return Err(BoxedErr::new(TextureError::UnsupportedSize {
            max: (u16::max_value() as usize, u16::max_value() as usize),
            got: (image.w, image.h),
        }));
    }

    let tb = apply_options(
        TextureBuilder::new(image.buf)
            .with_format(fmt)
            .with_size(image.w as u16, image.h as u16),
        options,
    );

    renderer.create_texture(tb).map_err(BoxedErr::new)
}
