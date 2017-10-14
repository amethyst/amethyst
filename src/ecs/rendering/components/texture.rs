//! Texture resource handling.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use gfx::format::SurfaceType;
use imagefmt::ColFmt;
use renderer::{Error as RendererError, Texture, TextureBuilder};

use assets::{ BoxedErr};
use renderer::formats::ImageData;

/// Error that can occur during texture creation
#[derive(Debug)]
pub enum TextureError {
    /// Error occured in renderer
    Renderer(RendererError),

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

/// Creates a texture asset.
pub fn create_texture_asset(
    image: ImageData,
    renderer: &mut ::renderer::Renderer,
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

    let tb = TextureBuilder::new(image.buf)
        .with_format(fmt)
        .with_size(image.w as u16, image.h as u16);
    renderer
        .create_texture(tb)
        .map_err(BoxedErr::new)
}
