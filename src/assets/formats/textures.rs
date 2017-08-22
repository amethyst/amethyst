//! Provides texture formats

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread::JoinHandle;

use assets::Format;
use futures::Future;
use image::{ImageBuffer, Rgba};
pub use image::ImageError;

pub struct ImageData {
    raw: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

pub struct ImageFuture {
    signal: Arc<AtomicBool>,
    handle: JoinHandle<ImageData>,
}

impl Future for ImageFuture {
    type Item = ImageData;
    type Error = ImageError;
}

pub struct JpegFormat;

impl Format for JpegFormat {
    type Data = ImageData;
    type Error = ImageError;
    type Result = ImageFuture;
}

pub struct JpgFormat;
