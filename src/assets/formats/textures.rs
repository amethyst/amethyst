//! Provides texture formats
//!

pub use imagefmt::Error as ImageError;

use std::io::Cursor;

use imagefmt;
use imagefmt::{ColFmt, Image};
use rayon::ThreadPool;

use assets::{Format, SpawnedFuture};

/// ImageData provided by formats, can be interpreted as a texture.
#[derive(Clone, Debug)]
pub struct ImageData {
    pub(crate) raw: Image<u8>,
}

/// A future which will eventually have an image available.
pub type ImageFuture = SpawnedFuture<ImageData, ImageError>;

/// Allows loading of jpg or jpeg files.
pub struct JpgFormat;

impl Format for JpgFormat {
    const EXTENSIONS: &'static [&'static str] = &["jpg", "jpeg"];
    type Data = ImageData;
    type Error = ImageError;
    type Result = ImageFuture;

    fn parse(&self, bytes: Vec<u8>, pool: &ThreadPool) -> Self::Result {
        ImageFuture::spawn(pool, move || {
            imagefmt::jpeg::read(&mut Cursor::new(bytes), ColFmt::RGBA).map(|raw| ImageData { raw })
        })
    }
}

/// Allows loading of PNG files.
pub struct PngFormat;

impl Format for PngFormat {
    const EXTENSIONS: &'static [&'static str] = &["png"];
    type Data = ImageData;
    type Error = ImageError;
    type Result = ImageFuture;

    fn parse(&self, bytes: Vec<u8>, pool: &ThreadPool) -> Self::Result {
        ImageFuture::spawn(pool, move || {
            imagefmt::png::read(&mut Cursor::new(bytes), ColFmt::RGBA).map(|raw| ImageData { raw })
        })
    }
}

/// Allows loading of BMP files.
pub struct BmpFormat;

impl Format for BmpFormat {
    const EXTENSIONS: &'static [&'static str] = &["bmp"];
    type Data = ImageData;
    type Error = ImageError;
    type Result = ImageFuture;

    fn parse(&self, bytes: Vec<u8>, pool: &ThreadPool) -> Self::Result {
        ImageFuture::spawn(pool, move || {
            imagefmt::bmp::read(&mut Cursor::new(bytes), ColFmt::RGBA).map(|raw| ImageData { raw })
        })
    }
}
