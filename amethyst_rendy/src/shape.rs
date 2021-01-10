//! 3D Shape Primitives
use amethyst_assets::{DefaultLoader, Handle, Loader, ProcessingQueue, Progress};
use amethyst_core::math::Vector3;
use genmesh::{
    generators::{
        Circle, Cone, Cube, Cylinder, IcoSphere, IndexedPolygon, Plane, SharedVertex, SphereUv,
        Torus,
    },
    EmitTriangles, MapVertex, Triangulate, Vertex, Vertices,
};
use rendy::mesh::{
    MeshBuilder, Normal, PosNormTangTex, PosNormTex, PosTex, Position, Tangent, TexCoord,
};

use crate::types::{Mesh, MeshData};
fn option_none<T>() -> Option<T> {
    None
}

/// Shape generators
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Shape {
    /// Sphere, number of points around the equator, number of points pole to pole.
    /// Sphere has radius of 1, so it's diameter is 2 units
    Sphere(usize, usize),
    /// Cone, number of subdivisions around the radius, must be > 1
    Cone(usize),
    /// Cube with vertices in [-1, +1] range, so it's width is 2 units
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

/// Required resource access to upload shape
#[allow(missing_debug_implementations)]
pub struct ShapeUpload<'a> {
    loader: &'a DefaultLoader,
    storage: &'a mut ProcessingQueue<MeshData>,
}

/// Vertex data for a basic shape.
pub type InternalVertexData = ([f32; 3], [f32; 3], [f32; 2], [f32; 3]);

/// Internal Shape, used for transformation from `genmesh` to `MeshBuilder`
#[derive(Debug)]
pub struct InternalShape(Vec<InternalVertexData>);

impl InternalShape {
    fn map_into<T, F: FnMut(&InternalVertexData) -> T>(&self, f: F) -> Vec<T> {
        self.0.iter().map(f).collect()
    }
}

/// Trait for providing conversion from a basic shape type.
pub trait FromShape {
    /// Convert from a shape to `Self` type.
    fn from(shape: &InternalShape) -> Self;
}

/// Internal trait for converting from vertex data to a shape type.
pub trait FromInternalVertex {
    /// Convert from a set of vertex data to `Self` type.
    fn from_internal(v: &InternalVertexData) -> Self;
}

impl<T: FromInternalVertex> FromShape for Vec<T> {
    fn from(shape: &InternalShape) -> Self {
        shape.map_into(T::from_internal)
    }
}

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
    ///
    /// - `Vec<PosTex>`
    /// - `Vec<PosNormTex>`
    /// - `Vec<PosNormTangTex>`
    /// - `ComboMeshCreator`
    /// `P`: Progress tracker type
    pub fn upload<V, P>(
        &self,
        scale: Option<(f32, f32, f32)>,
        upload: &ShapeUpload<'_>,
        progress: P,
    ) -> Handle<Mesh>
    where
        V: FromShape + Into<MeshBuilder<'static>>,
        P: Progress,
    {
        upload
            .loader
            .load_from_data(self.generate::<V>(scale).into(), progress, &upload.storage)
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
    ///
    /// - `Vec<PosTex>`
    /// - `Vec<PosNormTex>`
    /// - `Vec<PosNormTangTex>`
    /// - `ComboMeshCreator`
    pub fn generate<V>(&self, scale: Option<(f32, f32, f32)>) -> MeshBuilder<'static>
    where
        V: FromShape + Into<MeshBuilder<'static>>,
    {
        V::from(&self.generate_internal(scale)).into()
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
    ///
    /// - `Vec<PosTex>`
    /// - `Vec<PosNormTex>`
    /// - `Vec<PosNormTangTex>`
    /// - `ComboMeshCreator`
    pub fn generate_vertices<V>(&self, scale: Option<(f32, f32, f32)>) -> V
    where
        V: FromShape,
    {
        V::from(&self.generate_internal(scale))
    }

    fn generate_internal(&self, scale: Option<(f32, f32, f32)>) -> InternalShape {
        let vertices = match *self {
            Shape::Cube => generate_vertices(Cube::new(), scale),
            Shape::Sphere(u, v) => generate_vertices(SphereUv::new(u, v), scale),
            Shape::Cone(u) => generate_vertices(Cone::new(u), scale),
            Shape::Cylinder(u, h) => {
                generate_vertices(
                    h.map(|h| Cylinder::subdivide(u, h))
                        .unwrap_or_else(|| Cylinder::new(u)),
                    scale,
                )
            }
            Shape::IcoSphere(divide) => {
                generate_vertices(
                    divide
                        .map(IcoSphere::subdivide)
                        .unwrap_or_else(IcoSphere::new),
                    scale,
                )
            }
            Shape::Torus(radius, tube_radius, radial_segments, tubular_segments) => {
                generate_vertices(
                    Torus::new(radius, tube_radius, radial_segments, tubular_segments),
                    scale,
                )
            }
            Shape::Plane(divide) => {
                generate_vertices(
                    divide
                        .map(|(x, y)| Plane::subdivide(x, y))
                        .unwrap_or_else(Plane::new),
                    scale,
                )
            }
            Shape::Circle(u) => generate_vertices(Circle::new(u), scale),
        };
        InternalShape(vertices)
    }
}

fn generate_vertices<F, P, G>(
    generator: G,
    scale: Option<(f32, f32, f32)>,
) -> Vec<InternalVertexData>
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
                    .unwrap_or_else(|| Vector3::new(v.pos.x, v.pos.y, v.pos.z));
                let normal = scale
                    .map(|(x, y, z)| {
                        Vector3::new(v.normal.x * x, v.normal.y * y, v.normal.z * z).normalize()
                    })
                    .unwrap_or_else(|| Vector3::new(v.normal.x, v.normal.y, v.normal.z));
                let tangent1 = normal.cross(&Vector3::x());
                let tangent2 = normal.cross(&Vector3::y());
                let tangent = if tangent1.norm_squared() > tangent2.norm_squared() {
                    tangent1
                } else {
                    tangent2
                }
                .cross(&normal);

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

impl FromInternalVertex for Position {
    fn from_internal(v: &InternalVertexData) -> Self {
        Position([v.0[0], v.0[1], v.0[2]])
    }
}

impl FromInternalVertex for TexCoord {
    fn from_internal(v: &InternalVertexData) -> Self {
        TexCoord([v.2[0], v.2[1]])
    }
}

impl FromInternalVertex for Normal {
    fn from_internal(v: &InternalVertexData) -> Self {
        Normal([v.1[0], v.1[1], v.1[2]])
    }
}

impl FromInternalVertex for Tangent {
    fn from_internal(v: &InternalVertexData) -> Self {
        Tangent([v.3[0], v.3[1], v.3[2], 1.0])
    }
}

macro_rules! impl_interleaved {
    ($($type:ident { $($member:ident),*}),*,) => {
        $(impl FromInternalVertex for $type {
            fn from_internal(v: &InternalVertexData) -> Self {
                Self {
                    $($member: FromInternalVertex::from_internal(v),)*
                }
            }
        })*
    }
}

impl_interleaved! {
    PosTex { position, tex_coord },
    PosNormTex { position, normal, tex_coord },
    PosNormTangTex { position, normal, tangent, tex_coord },
}

macro_rules! impl_nested_from {
    ($($from:ident),*) => {
        impl<$($from,)*> FromShape for ($($from,)*)
        where
            $($from: FromShape,)*
        {
            fn from(shape: &InternalShape) -> Self {
                ($($from::from(shape),)*)
            }
        }
    }
}

impl_nested_from!(A);
impl_nested_from!(A, B);
impl_nested_from!(A, B, C);
impl_nested_from!(A, B, C, D);
impl_nested_from!(A, B, C, D, E);
impl_nested_from!(A, B, C, D, E, F);
impl_nested_from!(A, B, C, D, E, F, G);
impl_nested_from!(A, B, C, D, E, F, G, H);
impl_nested_from!(A, B, C, D, E, F, G, H, I);
impl_nested_from!(A, B, C, D, E, F, G, H, I, J);
impl_nested_from!(A, B, C, D, E, F, G, H, I, J, K);
impl_nested_from!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_nested_from!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_nested_from!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_nested_from!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_nested_from!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

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
