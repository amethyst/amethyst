use crate::types::Mesh;
use amethyst_assets::{AssetStorage, Handle, Loader, PrefabData, Progress, ProgressCounter};
use amethyst_core::{
    ecs::prelude::{Entity, Read, ReadExpect, WriteStorage},
    math::Vector3,
};
use amethyst_error::Error;
use genmesh::{
    generators::{
        Circle, Cone, Cube, Cylinder, IcoSphere, IndexedPolygon, Plane, SharedVertex, SphereUv,
        Torus,
    },
    EmitTriangles, MapVertex, Triangulate, Vertex, Vertices,
};
use rendy::{
    hal::Backend,
    mesh::{MeshBuilder, Normal, PosNormTangTex, PosNormTex, PosTex, Position, Tangent, TexCoord},
};
use std::marker::PhantomData;

fn option_none<T>() -> Option<T> {
    None
}

/// Prefab for generating `Mesh` from basic shapes
///
/// ### Type parameters:
///
/// `V`: Vertex format to use, must be one of:
///     * `Vec<PosTex>`
///     * `Vec<PosNormTex>`
///     * `Vec<PosNormTangTex>`
///     * `ComboMeshCreator`
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ShapePrefab<B: Backend, V> {
    #[serde(skip)]
    #[serde(default = "option_none")]
    handle: Option<Handle<Mesh<B>>>,
    shape: Shape,
    #[serde(default)]
    shape_scale: Option<(f32, f32, f32)>,
    #[serde(skip)]
    _m: PhantomData<V>,
}

impl<'a, B, V> PrefabData<'a> for ShapePrefab<B, V>
where
    V: From<InternalShape> + Into<MeshBuilder<'static>>,
    B: Backend,
{
    type SystemData = (
        ReadExpect<'a, Loader>,
        WriteStorage<'a, Handle<Mesh<B>>>,
        Read<'a, AssetStorage<Mesh<B>>>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), Error> {
        let (_, ref mut meshes, _) = system_data;
        let self_handle = self.handle.as_ref().expect(
            "`ShapePrefab::load_sub_assets` was not called before `ShapePrefab::add_to_entity`",
        );
        meshes.insert(entity, self_handle.clone())?;
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut <Self as PrefabData<'_>>::SystemData,
    ) -> Result<bool, Error> {
        let (loader, _, mesh_storage) = system_data;
        self.handle = Some(loader.load_from_data(
            self.shape.generate::<V>(self.shape_scale),
            progress,
            &mesh_storage,
        ));
        Ok(true)
    }
}

/// Shape generators
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Shape {
    /// Sphere, number of points around the equator, number of points pole to pole
    Sphere(usize, usize),
    /// Cone, number of subdivisions around the radius, must be > 1
    Cone(usize),
    /// Cube
    Cube,
    /// Cylinder, number of points across the radius, optional subdivides along the height
    Cylinder(usize, Option<usize>),
    /// Torus, radius from origin to center of tubular, tubular radius from toridal to surface,
    /// number of tube segments >= 3, number of segments around the tube
    Torus(f32, f32, usize, usize),
    /// Icosahedral sphere, number of subdivisions > 0 if given
    IcoSphere(Option<usize>),
    /// Plane, located in the XY plane, number of subdivisions along x and y axis if given
    Plane(Option<(usize, usize)>),
    /// Circle, located in the XY plane, number of points around the circle
    Circle(usize),
}

/// `SystemData` needed to upload a `Shape` directly to create a `Handle<Mesh>`
#[derive(SystemData)]
pub struct ShapeUpload<'a, B: Backend> {
    loader: ReadExpect<'a, Loader>,
    storage: Read<'a, AssetStorage<Mesh<B>>>,
}

pub type VertexFormat = ([f32; 3], [f32; 3], [f32; 2], [f32; 3]);

/// Internal Shape, used for transformation from `genmesh` to `MeshBuilder`
#[derive(Debug)]
pub struct InternalShape(Vec<VertexFormat>);

impl Shape {
    /// Generate `Mesh` for the `Shape`, and convert it into a `Handle<Mesh>`.
    ///
    /// ### Parameters:
    ///
    /// - `scale`: Scale the shape by the given amounts along the x, y, z axes
    /// - `upload`: ECS resources needed for uploading the mesh
    /// - `progress`: Progress tracker
    ///
    /// ### Type parameters:
    ///
    /// `V`: Vertex format to use, must to be one of:
    ///     * `Vec<PosTex>`
    ///     * `Vec<PosNormTex>`
    ///     * `Vec<PosNormTangTex>`
    ///     * `ComboMeshCreator`
    pub fn upload<V, P, B: Backend>(
        &self,
        scale: Option<(f32, f32, f32)>,
        upload: ShapeUpload<'_, B>,
        progress: P,
    ) -> Handle<Mesh<B>>
    where
        V: From<InternalShape> + Into<MeshBuilder<'static>>,
        P: Progress,
    {
        upload
            .loader
            .load_from_data(self.generate::<V>(scale), progress, &upload.storage)
    }

    /// Generate `MeshBuilder` for the `Shape`
    ///
    /// ### Parameters:
    ///
    /// - `scale`: Scale the shape by the given amounts along the x, y, z axes
    ///
    /// ### Type parameters:
    ///
    /// `V`: Vertex format to use, must to be one of:
    ///     * `Vec<PosTex>`
    ///     * `Vec<PosNormTex>`
    ///     * `Vec<PosNormTangTex>`
    ///     * `ComboMeshCreator`
    pub fn generate<V>(&self, scale: Option<(f32, f32, f32)>) -> MeshBuilder<'static>
    where
        V: From<InternalShape> + Into<MeshBuilder<'static>>,
    {
        V::from(self.generate_internal(scale)).into()
    }

    /// Generate vertices for the `Shape`, in format `V`
    ///
    /// ### Parameters:
    ///
    /// - `scale`: Scale the shape by the given amounts along the x, y, z axes
    ///
    /// ### Type parameters:
    ///
    /// `V`: Vertex format to use, must to be one of:
    ///     * `Vec<PosTex>`
    ///     * `Vec<PosNormTex>`
    ///     * `Vec<PosNormTangTex>`
    ///     * `ComboMeshCreator`
    pub fn generate_vertices<V>(&self, scale: Option<(f32, f32, f32)>) -> V
    where
        V: From<InternalShape>,
    {
        V::from(self.generate_internal(scale))
    }

    fn generate_internal(&self, scale: Option<(f32, f32, f32)>) -> InternalShape {
        let vertices = match *self {
            Shape::Cube => generate_vertices(Cube::new(), scale),
            Shape::Sphere(u, v) => generate_vertices(SphereUv::new(u, v), scale),
            Shape::Cone(u) => generate_vertices(Cone::new(u), scale),
            Shape::Cylinder(u, h) => generate_vertices(
                h.map(|h| Cylinder::subdivide(u, h))
                    .unwrap_or_else(|| Cylinder::new(u)),
                scale,
            ),
            Shape::IcoSphere(divide) => generate_vertices(
                divide
                    .map(IcoSphere::subdivide)
                    .unwrap_or_else(IcoSphere::new),
                scale,
            ),
            Shape::Torus(radius, tube_radius, radial_segments, tubular_segments) => {
                generate_vertices(
                    Torus::new(radius, tube_radius, radial_segments, tubular_segments),
                    scale,
                )
            }
            Shape::Plane(divide) => generate_vertices(
                divide
                    .map(|(x, y)| Plane::subdivide(x, y))
                    .unwrap_or_else(Plane::new),
                scale,
            ),
            Shape::Circle(u) => generate_vertices(Circle::new(u), scale),
        };
        InternalShape(vertices)
    }
}

fn generate_vertices<F, P, G>(generator: G, scale: Option<(f32, f32, f32)>) -> Vec<VertexFormat>
where
    F: EmitTriangles<Vertex = Vertex>,
    F::Vertex: Clone + Copy + PartialEq,
    P: EmitTriangles<Vertex = usize>,
    G: SharedVertex<F::Vertex> + IndexedPolygon<P> + Iterator<Item = F>,
{
    let vertices = generator.shared_vertex_iter().collect::<Vec<_>>();
    generator
        .indexed_polygon_iter()
        .triangulate()
        .map(|f| {
            f.map_vertex(|u| {
                let v = vertices[u];
                let pos = scale
                    .map(|(x, y, z)| Vector3::new(v.pos.x * x, v.pos.y * y, v.pos.z * z))
                    .unwrap_or_else(|| Vector3::from(v.pos));
                let normal = scale
                    .map(|(x, y, z)| {
                        Vector3::new(v.normal.x * x, v.normal.y * y, v.normal.z * z).normalize()
                    })
                    .unwrap_or_else(|| Vector3::from(v.normal));
                let up = Vector3::y();
                let tangent = normal.cross(&up).cross(&normal);
                (
                    pos.into(),
                    normal.into(),
                    [(v.pos.x + 1.) / 2., (v.pos.y + 1.) / 2.],
                    tangent.into(),
                )
            })
        })
        .vertices()
        .collect::<Vec<_>>()
}

impl From<InternalShape> for Vec<PosTex> {
    fn from(shape: InternalShape) -> Self {
        shape
            .0
            .iter()
            .map(|v| PosTex {
                position: Position([v.0[0], v.0[1], v.0[2]]),
                tex_coord: TexCoord([v.2[0], v.2[1]]),
            })
            .collect()
    }
}

impl From<InternalShape> for Vec<PosNormTex> {
    fn from(shape: InternalShape) -> Self {
        shape
            .0
            .iter()
            .map(|v| PosNormTex {
                position: Position([v.0[0], v.0[1], v.0[2]]),
                tex_coord: TexCoord([v.2[0], v.2[1]]),
                normal: Normal([v.1[0], v.1[1], v.1[2]]),
            })
            .collect()
    }
}

impl From<InternalShape> for Vec<PosNormTangTex> {
    fn from(shape: InternalShape) -> Self {
        shape
            .0
            .iter()
            .map(|v| PosNormTangTex {
                position: Position([v.0[0], v.0[1], v.0[2]]),
                tex_coord: TexCoord([v.2[0], v.2[1]]),
                normal: Normal([v.1[0], v.1[1], v.1[2]]),
                tangent: Tangent([v.3[0], v.3[1], v.3[2]]),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane() {
        println!(
            "{:?}",
            Shape::Plane(None).generate::<Vec<PosNormTangTex>>(None)
        );
    }
}
