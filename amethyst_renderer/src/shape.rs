use amethyst_core::cgmath::{InnerSpace, Vector3};
use genmesh::generators::{Cone, Cube, Cylinder, IcoSphere, IndexedPolygon, Plane, SharedVertex,
                          SphereUv, Torus};
use genmesh::{EmitTriangles, MapVertex, Triangulate, Vertex, Vertices};

use {ComboMeshCreator, MeshData, Normal, PosNormTangTex, PosNormTex, PosTex, Position, Separate,
     Tangent, TexCoord};

/// Shape generators
#[derive(Clone, Debug, Serialize, Deserialize)]
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
}

pub type VertexFormat = ([f32; 3], [f32; 3], [f32; 2], [f32; 3]);

/// Internal Shape, used for transformation from `genmesh` to `MeshData`
#[derive(Debug)]
pub struct InternalShape(Vec<VertexFormat>);

impl Shape {
    /// Generate `MeshData` for the `Shape`
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
    pub fn generate<V>(&self, scale: Option<(f32, f32, f32)>) -> MeshData
    where
        V: From<InternalShape> + Into<MeshData>,
    {
        V::from(self.generate_internal(scale)).into()
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
                    .map(|d| IcoSphere::subdivide(d))
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
                let up = Vector3::from([0.0, 1.0, 0.0]);
                let tangent = normal.cross(up).cross(normal);
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
                position: v.0,
                tex_coord: v.2,
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
                position: v.0,
                tex_coord: v.2,
                normal: v.1,
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
                position: v.0,
                tex_coord: v.2,
                normal: v.1,
                tangent: v.3,
            })
            .collect()
    }
}

impl From<InternalShape> for ComboMeshCreator {
    fn from(shape: InternalShape) -> Self {
        ComboMeshCreator::new((
            shape
                .0
                .iter()
                .map(|v| Separate::<Position>::new(v.0))
                .collect(),
            None,
            Some(
                shape
                    .0
                    .iter()
                    .map(|v| Separate::<TexCoord>::new(v.2))
                    .collect(),
            ),
            Some(
                shape
                    .0
                    .iter()
                    .map(|v| Separate::<Normal>::new(v.1))
                    .collect(),
            ),
            Some(
                shape
                    .0
                    .iter()
                    .map(|v| Separate::<Tangent>::new(v.3))
                    .collect(),
            ),
        ))
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
