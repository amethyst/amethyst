use std::fmt::Debug;

use amethyst_assets::{Asset, AssetStorage, SimpleFormat, Error as AssetsError, Handle};
use core::cgmath::{InnerSpace, Vector3};
use hal::Backend;
use specs::{Component, DenseVecStorage, VecStorage};
use wavefront_obj::obj::{parse, Normal, NormalIndex, ObjSet, Object, Primitive, TVertex,
                         TextureIndex, Vertex, VertexIndex};


use {Error};
use factory::Factory;
use mesh::{Mesh, MeshBuilder};
use vertex::{self, PosColor, PosNormTangTex, PosNormTex, PosTex};

/// Vertex combo
pub type VertexBufferCombination = (
    Vec<vertex::Position>,
    Option<Vec<vertex::Color>>,
    Option<Vec<vertex::TexCoord>>,
    Option<Vec<vertex::Normal>>,
    Option<Vec<vertex::Tangent>>,
);

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

    /// Combination of separate attributes
    Combination(VertexBufferCombination),

    /// Create a mesh from a given builder
    Builder(MeshBuilder<'static>),
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

impl From<VertexBufferCombination> for MeshData {
    fn from(data: VertexBufferCombination) -> Self {
        MeshData::Combination(data)
    }
}

/// Allows loading from Wavefront files
/// see: https://en.wikipedia.org/wiki/Wavefront_.obj_file
#[derive(Clone)]
pub struct ObjFormat;

impl<B> SimpleFormat<Mesh<B>> for ObjFormat
where
    B: Backend,
{
    const NAME: &'static str = "WAVEFRONT_OBJ";

    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<MeshData, AssetsError> {
        String::from_utf8(bytes)
            .map_err(Into::into)
            .and_then(|string| {
                parse(string)
                    .map_err(|e| {
                        AssetsError::from(format!("Failed to parse OBJ. Error in line {}: {:?}", e.line_number, e.message))
                    })
            })
            .map(|set| from_data(set).into())
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
            vertex::Position([vertex.x as f32, vertex.y as f32, vertex.z as f32])
        },
        normal: ni.map(|i| {
            let normal: Normal = object.normals[i];
            Vector3::from([normal.x as f32, normal.y as f32, normal.z as f32])
                .normalize()
                .into()
        }).unwrap_or([0.0, 0.0, 0.0])
            .into(),
        tex_coord: ti.map(|i| {
            let tvertex: TVertex = object.tex_vertices[i];
            [tvertex.u as f32, tvertex.v as f32]
        }).unwrap_or([0.0, 0.0])
            .into(),
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

macro_rules! build_mesh_with_some {
    ($builder:expr, $($args:expr),+ => { $h:expr $(,$t:expr)* }) => {
        match $h {
            Some(ref vertices) => build_mesh_with_some!($builder.with_vertices(&vertices[..]),
                                                    $($args),+ => {$($t),*}),
            None => build_mesh_with_some!($builder, $($args),+ => {$($t),*}),
        }
    };

    ($builder:expr, $($args:expr),+ => {}) => {
        $builder.build($($args),+)
    };
}

/// Create mesh
pub fn create_mesh_asset<B>(
    data: &MeshData,
    factory: &mut Factory<B>,
) -> Result<Mesh<B>, Error>
where
    B: Backend,
{
    match *data {
        MeshData::PosColor(ref vertices) => {
            let mb = MeshBuilder::new().with_vertices(&vertices[..]);
            mb.build(factory)
        }
        MeshData::PosTex(ref vertices) => {
            let mb = MeshBuilder::new().with_vertices(&vertices[..]);
            mb.build(factory)
        }
        MeshData::PosNormTex(ref vertices) => {
            let mb = MeshBuilder::new().with_vertices(&vertices[..]);
            mb.build(factory)
        }
        MeshData::PosNormTangTex(ref vertices) => {
            let mb = MeshBuilder::new().with_vertices(&vertices[..]);
            mb.build(factory)
        }
        MeshData::Combination(ref combo) => build_mesh_with_combo(combo, factory),
        MeshData::Builder(ref builder) => builder.build(factory),
    }.map_err(|err| Error::with_chain(err, format!("Failed to build mesh")))
}

/// Build Mesh with vertex buffer combination
pub fn build_mesh_with_combo<B>(
    combo: &VertexBufferCombination,
    factory: &mut Factory<B>,
) -> Result<Mesh<B>, Error>
where
    B: Backend,
{
    build_mesh_with_some!(
        MeshBuilder::new().with_vertices(&combo.0[..]), factory => {combo.1, combo.2, combo.3, combo.4}
    )
}

/// A handle to a mesh.
pub type MeshHandle<B: Backend> = Handle<Mesh<B>>;

impl<B> Asset for Mesh<B>
where
    B: Backend,
{
    const NAME: &'static str = "Mesh";
    type Data = MeshData;
    type HandleStorage = DenseVecStorage<MeshHandle<B>>;
}
