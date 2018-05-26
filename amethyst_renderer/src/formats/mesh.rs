use std::fmt;
use std::result::Result as StdResult;

use amethyst_assets::{Asset, SimpleFormat};
use amethyst_core::cgmath::{InnerSpace, Vector3};
use amethyst_core::specs::prelude::VecStorage;
use failure::{Error, ResultExt};
use wavefront_obj::obj::{parse, Normal, NormalIndex, ObjSet, Object, Primitive, TVertex,
                         TextureIndex, Vertex, VertexIndex};

use mesh::{Mesh, MeshBuilder, MeshHandle};
use vertex::*;
use {ErrorKind, Renderer, Result};

mod error {
    use std::fmt;
    use failure::{Fail, Context, Backtrace};

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Fail)]
    pub enum MeshImportErrorKind {
        /// An error occurred during parsing a mesh format
        #[fail(display="An error occurred during parsing a mesh format")]
        Parse,
        /// An unexpected error occurred during mesh import
        #[fail(display="An unexpected error occurred during mesh import")]
        Other,
    }

    #[derive(Debug)]
    pub struct MeshImportError {
        inner: Context<MeshImportErrorKind>
    }

    impl Fail for MeshImportError {
        fn cause(&self) -> Option<&Fail> {
            self.inner.cause()
        }

        fn backtrace(&self) -> Option<&Backtrace> {
            self.inner.backtrace()
        }
    }

    impl fmt::Display for MeshImportError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Display::fmt(&self.inner, f)
        }
    }

    impl MeshImportError {
        /// Get the kind of this error.
        pub fn kind(&self) -> &MeshImportErrorKind {
            self.inner.get_context()
        }
    }

    impl From<MeshImportErrorKind> for MeshImportError {
        fn from(kind: MeshImportErrorKind) -> Self {
            MeshImportError {
                inner: Context::new(kind),
            }
        }
    }

    impl From<Context<MeshImportErrorKind>> for MeshImportError {
        fn from(inner: Context<MeshImportErrorKind>) -> Self {
            MeshImportError { inner }
        }
    }
}
pub use self::error::{MeshImportError, MeshImportErrorKind};

/// Mesh data for loading
#[derive(Debug)]
pub enum MeshData {
    /// Position and color
    PosColor(Vec<PosColor>),

    /// Position and texture coordinates
    PosTex(Vec<PosTex>),

    /// Position, normal and texture coordinates
    PosNormTex(Vec<PosNormTex>),

    /// Position, normal, tangent and texture coordinates
    PosNormTangTex(Vec<PosNormTangTex>),

    /// Create a mesh from a given creator
    Creator(Box<MeshCreator>),
}

impl From<Vec<PosColor>> for MeshData {
    fn from(data: Vec<PosColor>) -> Self {
        MeshData::PosColor(data)
    }
}

impl From<Vec<PosTex>> for MeshData {
    fn from(data: Vec<PosTex>) -> Self {
        MeshData::PosTex(data)
    }
}

impl From<Vec<PosNormTex>> for MeshData {
    fn from(data: Vec<PosNormTex>) -> Self {
        MeshData::PosNormTex(data)
    }
}

impl From<Vec<PosNormTangTex>> for MeshData {
    fn from(data: Vec<PosNormTangTex>) -> Self {
        MeshData::PosNormTangTex(data)
    }
}

impl<M> From<M> for MeshData
where
    M: MeshCreator,
{
    fn from(creator: M) -> Self {
        MeshData::Creator(Box::new(creator))
    }
}

impl Asset for Mesh {
    const NAME: &'static str = "renderer::Mesh";
    type Data = MeshData;
    type HandleStorage = VecStorage<MeshHandle>;
}

/// Allows loading from Wavefront files
/// see: https://en.wikipedia.org/wiki/Wavefront_.obj_file
#[derive(Clone)]
pub struct ObjFormat;

impl SimpleFormat<Mesh> for ObjFormat {
    const NAME: &'static str = "WAVEFRONT_OBJ";

    type Options = ();
    type Error = MeshImportError;

    fn import(&self, bytes: Vec<u8>, _options: ()) -> StdResult<MeshData, Self::Error> {
        let s = String::from_utf8(bytes).context(MeshImportErrorKind::Other)?;
        let set = parse(s).map_err(|e| {
            format_err!("In line {}: {:?}", e.line_number, e.message).context("Failed to parse OBJ")
        }).context(MeshImportErrorKind::Parse)?;
        Ok(from_data(set).into())
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

/// Create mesh
pub fn create_mesh_asset(data: MeshData, renderer: &mut Renderer) -> Result<Mesh> {
    let data = match data {
        MeshData::PosColor(ref vertices) => {
            let mb = MeshBuilder::new(vertices);
            renderer.create_mesh(mb)
        }
        MeshData::PosTex(ref vertices) => {
            let mb = MeshBuilder::new(vertices);
            renderer.create_mesh(mb)
        }
        MeshData::PosNormTex(ref vertices) => {
            let mb = MeshBuilder::new(vertices);
            renderer.create_mesh(mb)
        }
        MeshData::PosNormTangTex(ref vertices) => {
            let mb = MeshBuilder::new(vertices);
            renderer.create_mesh(mb)
        }
        MeshData::Creator(creator) => creator
            .build(renderer)
            .map_err(|e| e.context(ErrorKind::MeshCreation).into()),
    };

    data
}

/// Build Mesh with vertex buffer combination
pub fn build_mesh_with_combo(
    combo: VertexBufferCombination,
    renderer: &mut Renderer,
) -> Result<Mesh> {
    build_mesh_with_some!(
        MeshBuilder::new(combo.0),
        renderer,
        combo.1,
        combo.2,
        combo.3,
        combo.4
    )
}

/// Trait used by the asset processor to convert any user supplied mesh representation into an
/// actual `Mesh`.
///
/// This allows the user to create their own vertex attributes, and have the amethyst asset and
/// render systems be able to convert it into a `Mesh` that can be used from any applicable
/// pass.
pub trait MeshCreator: Send + Sync + fmt::Debug + 'static {
    /// Build a mesh given a `Renderer`
    fn build(self: Box<Self>, renderer: &mut Renderer) -> StdResult<Mesh, Error>;
}

/// Mesh creator for `VertexBufferCombination`.
#[derive(Debug)]
pub struct ComboMeshCreator {
    combo: VertexBufferCombination,
}

impl ComboMeshCreator {
    /// Create a new combo mesh creator with the given combo
    pub fn new(combo: VertexBufferCombination) -> Self {
        Self { combo }
    }
}

impl MeshCreator for ComboMeshCreator {
    fn build(self: Box<Self>, renderer: &mut Renderer) -> StdResult<Mesh, Error> {
        build_mesh_with_combo(self.combo, renderer).map_err(|e| e.into())
    }
}

impl From<VertexBufferCombination> for ComboMeshCreator {
    fn from(combo: VertexBufferCombination) -> Self {
        Self::new(combo)
    }
}
