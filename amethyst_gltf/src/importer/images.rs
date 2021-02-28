use amethyst_assets::{distill_importer::ImportedAsset, error::Error};
use gltf::{buffer::Data, image::Source};
use log::error;

use crate::importer::GltfImporterState;

#[derive(Debug)]
pub enum ImageFormat {
    Png,
    Jpeg,
}

impl ImageFormat {
    fn from_mime_type(mime: &str) -> Self {
        match mime {
            "image/jpeg" => ImageFormat::Jpeg,
            "image/png" => ImageFormat::Png,
            _ => unreachable!(),
        }
    }
}

pub fn _load_image(
    _image: &gltf::Image<'_>,
    _state: &mut GltfImporterState,
    _buffers: &Vec<Data>,
) -> Vec<ImportedAsset> {
    vec![]
}

pub fn read_image_data(
    image: &gltf::Image<'_>,
    buffers: &Vec<Data>,
) -> Result<(Vec<u8>, ImageFormat), Error> {
    match image.source() {
        Source::View { view, mime_type } => {
            if let Some(data) = buffers.get(view.buffer().index()) {
                let sliced = data.0.as_slice();
                let begin = view.offset();
                let end = begin + view.length();
                if sliced.len() > end {
                    Ok((
                        data[begin..end].to_vec(),
                        ImageFormat::from_mime_type(mime_type),
                    ))
                } else {
                    error!("Image loading didn't work");
                    Err(Error::Source)
                }
            } else {
                error!("Image loading didn't work");
                Err(Error::Source)
            }
        }
        Source::Uri { .. } => {
            // TODO
            Err(Error::Source)
        }
    }
}
