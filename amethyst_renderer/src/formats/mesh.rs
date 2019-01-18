use std::{fmt::Debug, result::Result as StdResult};

use amethyst_assets::{
    Asset, AssetStorage, Error, Loader, PrefabData, PrefabError, ProcessingState, Result,
    ResultExt, SimpleFormat,
};
use amethyst_core::{
    nalgebra::{Vector2, Vector3},
    specs::prelude::{Component, Entity, Read, ReadExpect, VecStorage, WriteStorage},
};

use serde::{Deserialize, Serialize};
use wavefront_obj::obj::{
    parse, Normal, NormalIndex, ObjSet, Object, Primitive, TVertex, TextureIndex, Vertex,
    VertexIndex,
};

use crate::{
    mesh::{Mesh, MeshBuilder, MeshHandle},
    vertex::*,
    Renderer,
};

/// Mesh data for loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeshData {
    /// Position and color
    PosColor(Vec<PosColor>),

    /// Position, color and normal
    PosColorNorm(Vec<PosColorNorm>),

    /// Position and texture coordinates
    PosTex(Vec<PosTex>),

    /// Position, normal and texture coordinates
    PosNormTex(Vec<PosNormTex>),

    /// Position, normal, tangent and texture coordinates
    PosNormTangTex(Vec<PosNormTangTex>),

    /// Create a mesh from a given creator
    #[serde(skip)]
    Creator(Box<dyn MeshCreator>),
}

impl Component for MeshData {
    type Storage = VecStorage<Self>;
}

impl From<Vec<PosColor>> for MeshData {
    fn from(data: Vec<PosColor>) -> Self {
        MeshData::PosColor(data)
    }
}

impl From<Vec<PosColorNorm>> for MeshData {
    fn from(data: Vec<PosColorNorm>) -> Self {
        MeshData::PosColorNorm(data)
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

impl<'a> PrefabData<'a> for MeshData {
    type SystemData = (
        ReadExpect<'a, Loader>,
        WriteStorage<'a, MeshHandle>,
        Read<'a, AssetStorage<Mesh>>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
    ) -> StdResult<(), PrefabError> {
        let handle = system_data
            .0
            .load_from_data(self.clone(), (), &system_data.2);
        system_data.1.insert(entity, handle).map(|_| ())
    }
}

/// Allows loading from Wavefront files
/// see: https://en.wikipedia.org/wiki/Wavefront_.obj_file
#[derive(Clone, Deserialize, Serialize)]
pub struct ObjFormat;

impl SimpleFormat<Mesh> for ObjFormat {
    const NAME: &'static str = "WAVEFRONT_OBJ";

    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<MeshData> {
        String::from_utf8(bytes)
            .map_err(Into::into)
            .and_then(|string| {
                parse(string)
                    .map_err(|e| Error::from(format!("In line {}: {:?}", e.line_number, e.message)))
                    .chain_err(|| "Failed to parse OBJ")
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
            Vector3::new(vertex.x as f32, vertex.y as f32, vertex.z as f32)
        },
        normal: ni
            .map(|i| {
                let normal: Normal = object.normals[i];
                Vector3::from([normal.x as f32, normal.y as f32, normal.z as f32]).normalize()
            })
            .unwrap_or(Vector3::new(0.0, 0.0, 0.0)),
        tex_coord: ti
            .map(|i| {
                let tvertex: TVertex = object.tex_vertices[i];
                Vector2::new(tvertex.u as f32, tvertex.v as f32)
            })
            .unwrap_or(Vector2::new(0.0, 0.0)),
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
pub fn create_mesh_asset(data: MeshData, renderer: &mut Renderer) -> Result<ProcessingState<Mesh>> {
    let data = match data {
        MeshData::PosColor(ref vertices) => {
            let mb = MeshBuilder::new(vertices);
            renderer.create_mesh(mb)
        }
        MeshData::PosColorNorm(ref vertices) => {
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
        MeshData::Creator(creator) => creator.build(renderer),
    };

    data.map(ProcessingState::Loaded)
        .chain_err(|| "Failed to build mesh")
}

/// Build Mesh with vertex buffer combination
pub fn build_mesh_with_combo(
    combo: VertexBufferCombination,
    renderer: &mut Renderer,
) -> crate::error::Result<Mesh> {
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
pub trait MeshCreator: Send + Sync + Debug + 'static {
    /// Build a mesh given a `Renderer`
    fn build(self: Box<Self>, renderer: &mut Renderer) -> crate::error::Result<Mesh>;

    /// Returns the vertices contained in the MeshCreator.
    fn vertices(&self) -> &Vec<Separate<Position>>;

    /// Clone a boxed version of this object
    fn box_clone(&self) -> Box<dyn MeshCreator>;
}

impl Clone for Box<dyn MeshCreator> {
    fn clone(&self) -> Box<dyn MeshCreator> {
        self.box_clone()
    }
}

/// Mesh creator for `VertexBufferCombination`.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    fn build(self: Box<Self>, renderer: &mut Renderer) -> crate::error::Result<Mesh> {
        build_mesh_with_combo(self.combo, renderer)
    }

    fn vertices(&self) -> &Vec<Separate<Position>> {
        &self.combo.0
    }

    fn box_clone(&self) -> Box<dyn MeshCreator> {
        Box::new((*self).clone())
    }
}

impl From<VertexBufferCombination> for ComboMeshCreator {
    fn from(combo: VertexBufferCombination) -> Self {
        Self::new(combo)
    }
}
