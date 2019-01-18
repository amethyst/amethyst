use std::{self, error::Error as StdError, fmt, path::Path, sync::Arc};

use gltf::{
    self,
    json::{self, validation},
    Gltf,
};

use crate::assets::{Error as AssetError, Result as AssetResult, Source as AssetSource};

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

/// Buffer data returned from `import`.
#[derive(Clone, Debug)]
pub struct Buffers(Vec<Vec<u8>>);

#[allow(unused)]
impl Buffers {
    /// Obtain the contents of a loaded buffer.
    pub fn buffer(&self, buffer: &gltf::Buffer<'_>) -> Option<&[u8]> {
        self.0.get(buffer.index()).map(Vec::as_slice)
    }

    /// Obtain the contents of a loaded buffer view.
    pub fn view(&self, view: &gltf::buffer::View<'_>) -> Option<&[u8]> {
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
pub fn import<P>(source: Arc<dyn AssetSource>, path: P) -> Result<(Gltf, Buffers), Error>
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

fn read_to_end<P: AsRef<Path>>(source: Arc<dyn AssetSource>, path: P) -> AssetResult<Vec<u8>> {
    let path = path.as_ref();
    source.load(
        path.to_str()
            .expect("Path contains invalid UTF-8 charcters"),
    )
}

fn parse_data_uri(uri: &str) -> Result<Vec<u8>, Error> {
    let encoded = uri.split(",").nth(1).expect("URI does not contain ','");
    let decoded = base64::decode(&encoded)?;
    Ok(decoded)
}

fn load_external_buffers(
    source: Arc<dyn AssetSource>,
    base_path: &Path,
    gltf: &Gltf,
    mut bin: Option<Vec<u8>>,
) -> Result<Vec<Vec<u8>>, Error> {
    use gltf::buffer::Source;
    let mut buffers = vec![];
    for (index, buffer) in gltf.buffers().enumerate() {
        let data = match buffer.source() {
            Source::Uri(uri) => {
                if uri.starts_with("data:") {
                    parse_data_uri(uri)?
                } else {
                    let path = base_path.parent().unwrap_or(Path::new("./")).join(uri);
                    read_to_end(source.clone(), &path)?
                }
            }
            Source::Bin => bin
                .take()
                .expect("`BIN` section of binary glTF file is empty or used by another buffer"),
        };

        if data.len() < buffer.length() {
            let path = json::Path::new().field("buffers").index(index);
            return Err(Error::BufferLength(path));
        }
        buffers.push(data);
    }
    Ok(buffers)
}

fn import_standard(
    data: &[u8],
    source: Arc<dyn AssetSource>,
    base_path: &Path,
) -> Result<(Gltf, Buffers), Error> {
    let gltf = Gltf::from_slice(data)?;
    let buffers = Buffers(load_external_buffers(source, base_path, &gltf, None)?);
    Ok((gltf, buffers))
}

fn import_binary(
    data: &[u8],
    source: Arc<dyn AssetSource>,
    base_path: &Path,
) -> Result<(Gltf, Buffers), Error> {
    let gltf::binary::Glb {
        header: _,
        json,
        bin,
    } = gltf::binary::Glb::from_slice(data)?;
    let gltf = Gltf::from_slice(&json)?;
    let bin = bin.map(|x| x.to_vec());
    let buffers = Buffers(load_external_buffers(source, base_path, &gltf, bin)?);
    Ok((gltf, buffers))
}

pub fn get_image_data(
    image: &gltf::Image<'_>,
    buffers: &Buffers,
    source: Arc<dyn AssetSource>,
    base_path: &Path,
) -> Result<(Vec<u8>, ImageFormat), Error> {
    use gltf::image::Source;
    match image.source() {
        Source::View { view, mime_type } => {
            let data = buffers
                .view(&view)
                .expect("`view` of image data points to a buffer which does not exist");
            Ok((data.to_vec(), ImageFormat::from_mime_type(mime_type)))
        }

        Source::Uri { uri, mime_type } => {
            if uri.starts_with("data:") {
                let data = parse_data_uri(uri)?;
                if let Some(ty) = mime_type {
                    Ok((data, ImageFormat::from_mime_type(ty)))
                } else {
                    let mimetype = uri
                        .split(',')
                        .nth(0)
                        .expect("Unreachable: `split` will always return at least one element")
                        .split(':')
                        .nth(1)
                        .expect("URI does not contain ':'")
                        .split(';')
                        .nth(0)
                        .expect("Unreachable: `split` will always return at least one element");
                    Ok((data, ImageFormat::from_mime_type(mimetype)))
                }
            } else {
                let path = base_path.parent().unwrap_or(Path::new("./")).join(uri);
                let data = source.load(
                    path.to_str()
                        .expect("Path contains invalid UTF-8 characters"),
                )?;
                if let Some(ty) = mime_type {
                    Ok((data, ImageFormat::from_mime_type(ty)))
                } else {
                    let ext = path
                        .extension()
                        .and_then(|s| s.to_str())
                        .map_or("".to_string(), |s| s.to_ascii_lowercase());
                    let format = match &ext[..] {
                        "jpg" | "jpeg" => ImageFormat::Jpeg,
                        "png" => ImageFormat::Png,
                        _ => unreachable!(),
                    };
                    Ok((data, format))
                }
            }
        }
    }
}

/// Error encountered when importing a glTF 2.0 asset.
#[allow(unused)]
#[derive(Debug)]
pub enum Error {
    /// A loaded glTF buffer is not of the required length.
    BufferLength(json::Path),

    /// Base 64 decoding error.
    Base64Decoding(base64::DecodeError),

    /// A glTF extension required by the asset has not been enabled by the user.
    ExtensionDisabled(String),

    /// A glTF extension required by the asset is not supported by the library.
    ExtensionUnsupported(String),

    /// The glTF version of the asset is incompatible with the importer.
    IncompatibleVersion(String),

    /// Standard I/O error.
    Io(std::io::Error),

    /// `gltf` crate error.
    Gltf(gltf::Error),

    /// Failure when deserializing .gltf or .glb JSON.
    MalformedJson(json::Error),

    /// The .gltf data is invalid.
    Validation(Vec<(json::Path, validation::Error)>),

    /// Asset error
    Asset(AssetError),
}

impl From<AssetError> for Error {
    fn from(err: AssetError) -> Self {
        Error::Asset(err)
    }
}

impl From<json::Error> for Error {
    fn from(err: json::Error) -> Error {
        Error::MalformedJson(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<Vec<(json::Path, validation::Error)>> for Error {
    fn from(errs: Vec<(json::Path, validation::Error)>) -> Error {
        Error::Validation(errs)
    }
}

impl From<gltf::Error> for Error {
    fn from(err: gltf::Error) -> Error {
        match err {
            gltf::Error::Validation(errs) => Error::Validation(errs),
            _ => Error::Gltf(err),
        }
    }
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Error {
        Error::Base64Decoding(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::error::Error;
        write!(f, "{}", self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            Base64Decoding(_) => "Base 64 decoding failed",
            BufferLength(_) => "Loaded buffer does not match required length",
            ExtensionDisabled(_) => "Asset requires a disabled extension",
            ExtensionUnsupported(_) => "Assets requires an unsupported extension",
            IncompatibleVersion(_) => "Asset is not glTF version 2.0",
            Io(_) => "I/O error",
            Gltf(_) => "Error from gltf crate",
            MalformedJson(_) => "Malformed .gltf / .glb JSON",
            Validation(_) => "Asset failed validation tests",
            Asset(_) => "Failed loading file from source",
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        use self::Error::*;
        match *self {
            MalformedJson(ref err) => Some(err),
            Io(ref err) => Some(err),
            _ => None,
        }
    }
}
