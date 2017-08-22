//! Provides texture formats

use std::mem::replace;
use std::sync::{Arc, Mutex};

use assets::Format;
use futures::{Async, Future};
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
pub struct ImageFuture {
    result: Arc<Mutex<Option<Result<ImageData, ImageError>>>>,
}

impl Future for ImageFuture {
    type Item = ImageData;
    type Error = ImageError;

    fn poll(&mut self) -> Result<Async<ImageData>, ImageError> {
        let mut lock = self.result.lock().unwrap();
        if lock.is_some() {
            if lock.as_ref().unwrap().is_ok() {
                let image_data = replace(&mut *lock, None);
                Ok(Async::Ready(image_data.unwrap().unwrap()))
            } else {
                let image_error = replace(&mut *lock, None);
                Err(image_error.unwrap().unwrap_err())
            }
        } else {
            Ok(Async::NotReady)
        }
    }
}

fn parse_jpeg(bytes: Vec<u8>, pool: &ThreadPool) -> ImageFuture {
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();
    pool.spawn(move || {
        let result = image::load_from_memory_with_format(&bytes, ImageFormat::JPEG)
            .map(|di| ImageData { raw: di.to_rgba() });
        *result_clone.lock().unwrap() = Some(result);
    });
    ImageFuture {
        result,
    }
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
