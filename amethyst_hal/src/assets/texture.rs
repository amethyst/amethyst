pub use imagefmt::Error as ImageError;

use std::io::Cursor;

use amethyst_assets::{Asset, Handle, SimpleFormat, Error as AssetsError};

use hal::{Backend, Device};
use hal::format::Format;
use hal::image::{AaMode, Kind, SamplerInfo, Level, Size};
use hal::memory::{Pod, cast_slice};

use imagefmt::{self, ColFmt, Image};
use specs::DenseVecStorage;

use {Error};
use factory::Factory;
use texture::{Texture, TextureBuilder};
use utils::cast_vec;

/// ImageData provided by formats, can be interpreted as a texture.
#[derive(Clone, Debug)]
pub struct ImageData {
    /// The raw image data.
    pub raw: Image<u8>,
}
/// Allows loading of jpg or jpeg files.
#[derive(Clone)]
pub struct JpgFormat;

impl<B> SimpleFormat<Texture<B>> for JpgFormat
where
    B: Backend,
{
    const NAME: &'static str = "JPEG";

    type Options = ();

    fn import(
        &self,
        bytes: Vec<u8>,
        _options: (),
    ) -> Result<TextureBuilder<'static>, AssetsError> {
        let image = imagefmt::jpeg::read(&mut Cursor::new(bytes), ColFmt::RGBA).map_err(|err| {
            AssetsError::with_chain(err, "Failed to load jpeg from bytestream")
        })?;
        Ok(TextureBuilder::new()
            .with_kind(Kind::D2(image.w as u16, image.h as u16, AaMode::Single))
            .with_format(Format::Rgba8Unorm)
            .with_raw_data(image.buf))
    }
}

/// Allows loading of PNG files.
#[derive(Clone)]
pub struct PngFormat;

impl<B> SimpleFormat<Texture<B>> for PngFormat
where
    B: Backend,
{
    const NAME: &'static str = "PNG";

    type Options = ();

    fn import(
        &self,
        bytes: Vec<u8>,
        _options: (),
    ) -> Result<TextureBuilder<'static>, AssetsError> {
        let image = imagefmt::png::read(&mut Cursor::new(bytes), ColFmt::RGBA).map_err(|err| {
            AssetsError::with_chain(err, "Failed to load png from bytestream")
        })?;
        Ok(TextureBuilder::new()
            .with_kind(Kind::D2(image.w as u16, image.h as u16, AaMode::Single))
            .with_format(Format::Rgba8Unorm)
            .with_raw_data(image.buf))
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

    type Options = ();

    fn import(
        &self,
        bytes: Vec<u8>,
        _options: (),
    ) -> Result<TextureBuilder<'static>, AssetsError> {
        let image = imagefmt::png::read(&mut Cursor::new(bytes), ColFmt::RGBA).map_err(|err| {
            AssetsError::with_chain(err, "Failed to load png from bytestream")
        })?;
        Ok(TextureBuilder::new()
            .with_kind(Kind::D2(image.w as u16, image.h as u16, AaMode::Single))
            .with_format(Format::Rgba8Unorm)
            .with_raw_data(image.buf))
    }
}

/// A handle to a texture.
pub type TextureHandle<B: Backend> = Handle<Texture<B>>;

impl<B> Asset for Texture<B>
where
    B: Backend,
{
    const NAME: &'static str = "Texture";
    type Data = TextureBuilder<'static>;
    type HandleStorage = DenseVecStorage<TextureHandle<B>>;
}
