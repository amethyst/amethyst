use std::path::Path;
use std::sync::Arc;

use assets::Source as AssetSource;
use base64;
use {Error, ErrorKind};
use failure::{err_msg, ResultExt};
use gltf;
use gltf::Gltf;
use gltf::json;
use gltf_utils::Source;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ImageFormat {
    Png,
    Jpeg,
}

impl ImageFormat {
    fn from_mime_type(mime: &str) -> Result<Self, Error> {
        match mime {
            "image/jpeg" => Ok(ImageFormat::Jpeg),
            "image/png" => Ok(ImageFormat::Png),
            _ => Err(ErrorKind::UnknownImageType(mime.to_owned()).into()),
        }
    }
}

/// Buffer data returned from `import`.
#[derive(Clone, Debug)]
pub struct Buffers(Vec<Vec<u8>>);

impl Source for Buffers {
    fn source_buffer(&self, buffer: &gltf::Buffer) -> &[u8] {
        &self.0[buffer.index()]
    }
}

#[allow(unused)]
impl Buffers {
    /// Obtain the contents of a loaded buffer.
    pub fn buffer(&self, buffer: &gltf::Buffer) -> Option<&[u8]> {
        self.0.get(buffer.index()).map(Vec::as_slice)
    }

    /// Obtain the contents of a loaded buffer view.
    pub fn view(&self, view: &gltf::buffer::View) -> Option<&[u8]> {
        self.buffer(&view.buffer()).map(|data| {
            let begin = view.offset();
            let end = begin + view.length();
            &data[begin..end]
        })
    }

    /// Take the loaded buffer data.
    pub fn take(self) -> Vec<Vec<u8>> {
        self.0
    }
}

/// Imports glTF 2.0
pub fn import<P>(source: Arc<AssetSource>, path: P) -> Result<(Gltf, Buffers), Error>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let data = read_to_end(source.clone(), path)?;
    if data.starts_with(b"glTF") {
        import_binary(&data, source, path)
    } else {
        import_standard(&data, source, path)
    }
}

fn read_to_end<P: AsRef<Path>>(source: Arc<AssetSource>, path: P) -> Result<Vec<u8>, Error> {
    let path = path.as_ref();
    let path = path.to_str()
        .ok_or(format_err!("cannot convert path \"{:?}\" to string", path))
        .context(ErrorKind::Importer)?;
    let data = source.load(path).context(ErrorKind::Importer)?;
    Ok(data)
}

fn parse_data_uri(uri: &str) -> Result<Vec<u8>, Error> {
    let encoded = uri.split(",").nth(1)
        .ok_or(err_msg("expected ',' in data uri not found"))
        .context(ErrorKind::Importer)?;
    let decoded = base64::decode(&encoded).context(ErrorKind::Importer)?;
    Ok(decoded)
}

fn load_external_buffers(
    source: Arc<AssetSource>,
    base_path: &Path,
    gltf: &Gltf,
    mut bin: Option<Vec<u8>>,
) -> Result<Vec<Vec<u8>>, Error> {
    let mut buffers = vec![];
    for (index, buffer) in gltf.buffers().enumerate() {
        let uri = buffer.uri();
        let data_res: Result<Vec<u8>, Error> = if uri == "#bin" {
            Ok(bin.take().expect("internal error: uri says binary data is not empty, but it is"))
        } else if uri.starts_with("data:") {
            parse_data_uri(uri)
        } else {
            let path = base_path.parent().unwrap_or(Path::new("./")).join(uri);
            read_to_end(source.clone(), &path)
        };
        let data = data_res?;

        if data.len() < buffer.length() {
            let path = json::Path::new().field("buffers").index(index);
            return Err(ErrorKind::BufferLength(path).into());
        }
        buffers.push(data);
    }
    Ok(buffers)
}

fn validate_standard(unvalidated: gltf::Unvalidated) -> Result<Gltf, Error> {
    Ok(unvalidated.validate_completely().context(ErrorKind::Importer)?)
}

fn validate_binary(unvalidated: gltf::Unvalidated, has_bin: bool) -> Result<Gltf, Error> {
    use gltf::json::validation::Error as Reason;

    let mut errs = vec![];
    {
        let json = unvalidated.as_json();
        for (index, buffer) in json.buffers.iter().enumerate() {
            let path = || json::Path::new().field("buffers").index(index).field("uri");
            match index {
                0 if has_bin => if buffer.uri.is_some() {
                    errs.push((path(), Reason::Missing));
                },
                _ if buffer.uri.is_none() => {
                    errs.push((path(), Reason::Missing));
                }
                _ => {}
            }
        }
    }

    if errs.is_empty() {
        Ok(unvalidated.validate_completely().context(ErrorKind::Importer)?)
    } else {
        Err(ErrorKind::Validation(errs).into())
    }
}

fn import_standard(
    data: &[u8],
    source: Arc<AssetSource>,
    base_path: &Path,
) -> Result<(Gltf, Buffers), Error> {
    let gltf_unvalidated = Gltf::from_slice(data).context(ErrorKind::Importer)?;
    let gltf = validate_standard(gltf_unvalidated)?;
    let buffers = load_external_buffers(source, base_path, &gltf, None)?;
    Ok((gltf, Buffers(buffers)))
}

fn import_binary(
    data: &[u8],
    source: Arc<AssetSource>,
    base_path: &Path,
) -> Result<(Gltf, Buffers), Error> {
    let gltf::Glb {
        header: _,
        json,
        bin,
    } = gltf::Glb::from_slice(data).context(ErrorKind::Importer)?;
    let unvalidated = Gltf::from_slice(json).context(ErrorKind::Importer)?;
    let bin = bin.map(|x| x.to_vec());
    let gltf = validate_binary(unvalidated, bin.is_some())?;
    let buffers = Buffers(load_external_buffers(source, base_path, &gltf, bin)?);
    Ok((gltf, buffers))
}

pub fn get_image_data(
    image: &gltf::Image,
    buffers: &Buffers,
    source: Arc<AssetSource>,
    base_path: &Path,
) -> Result<(Vec<u8>, ImageFormat), Error> {
    match image.data() {
        gltf::image::Data::View { view, mime_type } => {
            let data = buffers.view(&view).unwrap();
            Ok((data.to_vec(), ImageFormat::from_mime_type(mime_type)?))
        }

        gltf::image::Data::Uri { uri, mime_type } => {
            if uri.starts_with("data:") {
                let data = parse_data_uri(uri)?;
                if let Some(ty) = mime_type {
                    Ok((data, ImageFormat::from_mime_type(ty)?))
                } else {
                    let mimetype = uri.split(',')
                        .nth(0)
                        .unwrap()
                        .split(':')
                        .nth(1)
                        .unwrap()
                        .split(';')
                        .nth(0)
                        .unwrap();
                    Ok((data, ImageFormat::from_mime_type(mimetype)?))
                }
            } else {
                let path = base_path.parent().unwrap_or(Path::new("./")).join(uri);
                let data = source.load(path.to_str().unwrap())
                    .context(ErrorKind::Importer)?;
                if let Some(ty) = mime_type {
                    Ok((data, ImageFormat::from_mime_type(ty)?))
                } else {
                    let ext = path.extension()
                        .and_then(|s| s.to_str())
                        .map_or("".to_string(), |s| s.to_ascii_lowercase());
                    let format = match &ext[..] {
                        "jpg" | "jpeg" => ImageFormat::Jpeg,
                        "png" => ImageFormat::Png,
                        _ => return Err(ErrorKind::UnknownImageType(ext.clone()).into()),
                    };
                    Ok((data, format))
                }
            }
        }
    }
}

