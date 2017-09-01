use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::string::FromUtf8Error;

use cgmath::{InnerSpace, Vector3};
use rayon::ThreadPool;
use renderer::vertex::PosNormTex;
use wavefront_obj::ParseError;
use wavefront_obj::obj::{Normal, NormalIndex, Object, ObjSet, parse, Primitive, TVertex,
                         TextureIndex, Vertex, VertexIndex};

use assets::{Format, SpawnedFuture};

/// A future which will eventually have an vertices available.
pub type VerticesFuture<V> = SpawnedFuture<Vec<V>, ObjError>;

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

/// Allows loading from Wavefront files
/// see: https://en.wikipedia.org/wiki/Wavefront_.obj_file
pub struct ObjFormat;

impl Format for ObjFormat {
    const EXTENSIONS: &'static [&'static str] = &["obj"];
    type Data = Vec<PosNormTex>;
    type Error = ObjError;
    type Result = VerticesFuture<PosNormTex>;

    fn parse(&self, bytes: Vec<u8>, pool: &ThreadPool) -> Self::Result {
        VerticesFuture::spawn(pool, move || {
            String::from_utf8(bytes)
                .map_err(ObjError::Utf8)
                .and_then(|string| parse(string).map_err(ObjError::Parse))
                .map(|set| from_data(set))
        })
    }
}



fn convert(
    object: &Object,
    vi: VertexIndex,
    ti: Option<TextureIndex>,
    ni: Option<NormalIndex>,
) -> PosNormTex {
    PosNormTex {
        a_position: {
            let vertex: Vertex = object.vertices[vi];
            [vertex.x as f32, vertex.y as f32, vertex.z as f32]
        },
        a_normal: ni.map(|i| {
            let normal: Normal = object.normals[i];
            Vector3::from([normal.x as f32, normal.y as f32, normal.z as f32])
                .normalize()
                .into()
        }).unwrap_or([0.0, 0.0, 0.0]),
        a_tex_coord: ti.map(|i| {
            let tvertex: TVertex = object.tex_vertices[i];
            [tvertex.u as f32, tvertex.v as f32]
        }).unwrap_or([0.0, 0.0]),
    }
}

fn convert_primitive(object: &Object, prim: &Primitive) -> Option<[PosNormTex; 3]> {
    match *prim {
        Primitive::Triangle(v1, v2, v3) => {
            Some(
                [
                    convert(object, v1.0, v1.1, v1.2),
                    convert(object, v2.0, v2.1, v2.2),
                    convert(object, v3.0, v3.1, v3.2),
                ],
            )
        }
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
            geometry.shapes.iter().filter_map(move |s| {
                convert_primitive(object, &s.primitive)
            })
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
