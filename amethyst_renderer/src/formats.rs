//! Provides texture formats
//!

pub use imagefmt::Error as ImageError;

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::Cursor;
use std::string::FromUtf8Error;
use std::sync::Arc;

use amethyst_assets::{Asset, BoxedErr, Format, Source};
use cgmath::{InnerSpace, Vector3};
use imagefmt;
use imagefmt::{ColFmt, Image};
use wavefront_obj::ParseError;
use wavefront_obj::obj::{parse, Normal, NormalIndex, ObjSet, Object, Primitive, TVertex,
                         TextureIndex, Vertex, VertexIndex};

use mesh::Mesh;
use tex::Texture;
use vertex::PosNormTex;

impl Asset for Texture {
    type Data = ImageData;
}

/// ImageData provided by formats, can be interpreted as a texture.
#[derive(Clone, Debug)]
pub struct ImageData {
    /// The raw image data.
    pub raw: Image<u8>,
}
/// Allows loading of jpg or jpeg files.
pub struct JpgFormat;

impl Format<Texture> for JpgFormat {
    const NAME: &'static str = "JPEG";

    fn import(&self, name: String, source: Arc<Source>) -> Result<ImageData, BoxedErr> {
        imagefmt::jpeg::read(&mut Cursor::new(source.load(&name)?), ColFmt::RGBA)
            .map(|raw| ImageData { raw })
            .map_err(BoxedErr::new)
    }
}

/// Allows loading of PNG files.
pub struct PngFormat;

impl Format<Texture> for PngFormat {
    const NAME: &'static str = "JPEG";

    fn import(&self, name: String, source: Arc<Source>) -> Result<ImageData, BoxedErr> {
        imagefmt::png::read(&mut Cursor::new(source.load(&name)?), ColFmt::RGBA)
            .map(|raw| ImageData { raw })
            .map_err(BoxedErr::new)
    }
}

/// Allows loading of BMP files.
pub struct BmpFormat;

impl Format<Texture> for BmpFormat {
    const NAME: &'static str = "BMP";

    fn import(&self, name: String, source: Arc<Source>) -> Result<ImageData, BoxedErr> {
        imagefmt::bmp::read(&mut Cursor::new(source.load(&name)?), ColFmt::RGBA)
            .map(|raw| ImageData { raw })
            .map_err(BoxedErr::new)
    }
}

/// Error type of `ObjFormat`
#[derive(Debug)]
pub enum ObjError {
    /// Coundn't convert bytes to `String`
    Utf8(FromUtf8Error),
    /// Cound't parse obj file
    Parse(ParseError),
}

impl Error for ObjError {
    fn description(&self) -> &str {
        match *self {
            ObjError::Utf8(ref err) => err.description(),
            ObjError::Parse(_) => "Obj parsing error",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ObjError::Utf8(ref err) => Some(err),
            ObjError::Parse(_) => None,
        }
    }
}

impl Display for ObjError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match *self {
            ObjError::Utf8(ref err) => write!(fmt, "Obj file not a unicode: {:?}", err),
            ObjError::Parse(ref err) => write!(fmt, "Obj parsing error: {}", err.message),
        }
    }
}

impl Asset for Mesh {
    type Data = Vec<PosNormTex>;
}

/// Allows loading from Wavefront files
/// see: https://en.wikipedia.org/wiki/Wavefront_.obj_file
pub struct ObjFormat;

impl Format<Mesh> for ObjFormat {
    const NAME: &'static str = "WAVEFRONT_OBJ";

    fn import(&self, name: String, source: Arc<Source>) -> Result<Vec<PosNormTex>, BoxedErr> {
        String::from_utf8(source.load(&name)?)
            .map_err(ObjError::Utf8)
            .and_then(|string| parse(string).map_err(ObjError::Parse))
            .map(|set| from_data(set))
            .map_err(BoxedErr::new)
    }
}

fn convert(
    object: &Object,
    vi: VertexIndex,
    ti: Option<TextureIndex>,
    ni: Option<NormalIndex>,
) -> PosNormTex {
    PosNormTex {
        position: {
            let vertex: Vertex = object.vertices[vi];
            [vertex.x as f32, vertex.y as f32, vertex.z as f32]
        },
        normal: ni.map(|i| {
            let normal: Normal = object.normals[i];
            Vector3::from([normal.x as f32, normal.y as f32, normal.z as f32])
                .normalize()
                .into()
        }).unwrap_or([0.0, 0.0, 0.0]),
        tex_coord: ti.map(|i| {
            let tvertex: TVertex = object.tex_vertices[i];
            [tvertex.u as f32, tvertex.v as f32]
        }).unwrap_or([0.0, 0.0]),
    }
}

fn convert_primitive(object: &Object, prim: &Primitive) -> Option<[PosNormTex; 3]> {
    match *prim {
        Primitive::Triangle(v1, v2, v3) => Some([
            convert(object, v1.0, v1.1, v1.2),
            convert(object, v2.0, v2.1, v2.2),
            convert(object, v3.0, v3.1, v3.2),
        ]),
        _ => None,
    }
}

fn from_data(obj_set: ObjSet) -> Vec<PosNormTex> {
    // Takes a list of objects that contain geometries that contain shapes that contain
    // vertex/texture/normal indices into the main list of vertices, and converts to a
    // flat vec of `PosNormTex` objects.
    // TODO: Doesn't differentiate between objects in a `*.obj` file, treats
    // them all as a single mesh.
    let vertices = obj_set.objects.iter().flat_map(|object| {
        object.geometry.iter().flat_map(move |geometry| {
            geometry
                .shapes
                .iter()
                .filter_map(move |s| convert_primitive(object, &s.primitive))
        })
    });

    let mut result = Vec::new();
    for vvv in vertices {
        result.push(vvv[0]);
        result.push(vvv[1]);
        result.push(vvv[2]);
    }
    result
}
