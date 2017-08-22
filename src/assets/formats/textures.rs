//! Provides texture formats

use assets::{SpawnedFuture, Format};
use image;
use image::{ImageBuffer, ImageFormat, Rgba};
use rayon::ThreadPool;

pub use image::ImageError;

/// ImageData provided by formats, can be interpreted as a texture.
#[derive(Clone, Debug)]
pub struct ImageData {
    raw: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

/// A future which will eventually have an image available.
pub type ImageFuture = SpawnedFuture<ImageData, ImageError>;

fn parse_jpeg(bytes: Vec<u8>, pool: &ThreadPool) -> ImageFuture {
    ImageFuture::spawn(pool, move || {
        image::load_from_memory_with_format(&bytes, ImageFormat::JPEG)
            .map(|di| ImageData { raw: di.to_rgba() })
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
