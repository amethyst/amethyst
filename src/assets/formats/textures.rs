//! Provides texture formats
//!

pub use imagefmt::Error as ImageError;

use std::io::Cursor;

use imagefmt;
use imagefmt::{Image, ColFmt};
use rayon::ThreadPool;

use assets::{SpawnedFuture, Format};

/// ImageData provided by formats, can be interpreted as a texture.
#[derive(Clone, Debug)]
pub struct ImageData {
    pub(crate) raw: Image<u8>,
}

/// A future which will eventually have an image available.
pub type ImageFuture = SpawnedFuture<ImageData, ImageError>;

fn parse_jpeg(bytes: Vec<u8>, pool: &ThreadPool) -> ImageFuture {
    ImageFuture::spawn(pool, move || {
        imagefmt::jpeg::read(&mut Cursor::new(bytes), ColFmt::RGBA).map(|raw| ImageData { raw })
    })
}

/// Allows loading of Jpeg files.
pub struct JpegFormat;

impl Format for JpegFormat {
    type Data = ImageData;
    type Error = ImageError;
    type Result = ImageFuture;

    fn extension() -> &'static str {
        "jpeg"
    }

    fn parse(&self, bytes: Vec<u8>, pool: &ThreadPool) -> Self::Result {
        parse_jpeg(bytes, pool)
    }
}

/// Allows loading of Jpg files.
pub struct JpgFormat;

impl Format for JpgFormat {
    type Data = ImageData;
    type Error = ImageError;
    type Result = ImageFuture;

    fn extension() -> &'static str {
        "jpg"
    }

    fn parse(&self, bytes: Vec<u8>, pool: &ThreadPool) -> Self::Result {
        parse_jpeg(bytes, pool)
    }
}

/// Allows loading of PNG files.
pub struct PngFormat;

impl Format for PngFormat {
    type Data = ImageData;
    type Error = ImageError;
    type Result = ImageFuture;

    fn extension() -> &'static str {
        "png"
    }

    fn parse(&self, bytes: Vec<u8>, pool: &ThreadPool) -> Self::Result {
        ImageFuture::spawn(pool, move || {
            imagefmt::png::read(&mut Cursor::new(bytes), ColFmt::RGBA).map(|raw| ImageData { raw })
        })
    }
}

/// Allows loading of BMP files.
pub struct BmpFormat;

impl Format for BmpFormat {
    type Data = ImageData;
    type Error = ImageError;
    type Result = ImageFuture;

    fn extension() -> &'static str {
        "bmp"
    }

    fn parse(&self, bytes: Vec<u8>, pool: &ThreadPool) -> Self::Result {
        ImageFuture::spawn(pool, move || {
            imagefmt::bmp::read(&mut Cursor::new(bytes), ColFmt::RGBA).map(|raw| ImageData { raw })
        })
    }
}
