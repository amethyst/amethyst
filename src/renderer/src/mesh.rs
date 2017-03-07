//! Mesh resource.

use {Buffer, Factory, Slice, VertexFormat};
use cgmath::{Deg, Matrix4, Point3, Vector3, Transform};

/// Represents a triangle mesh.
pub struct Mesh<V: VertexFormat> {
    slice: Slice,
    transform: Matrix4<f32>,
    vert_buf: Buffer<V>,
}

impl<V: VertexFormat> Mesh<V> {
    /// Returns the mesh's vertex buffer and associated buffer slice.
    pub fn geometry(&self) -> (&Buffer<V>, &Slice) {
        (&self.vert_buf, &self.slice)
    }

    /// Returns the transformation matrix of the mesh.
    ///
    /// This four-by-four matrix applies translation, rotation, and scaling to
    /// the mesh. It is often referred to in the computer graphics industry as
    /// the "model matrix".
    pub fn transform(&self) -> Matrix4<f32> {
        self.transform
    }
}

/// Builds new meshes.
pub struct MeshBuilder<'a, V: 'a + VertexFormat> {
    factory: &'a mut Factory,
    transform: Matrix4<f32>,
    vertices: &'a [V],
}

impl<'a, V: 'a + VertexFormat> MeshBuilder<'a, V> {
    /// Creates a new MeshBuilder with the given factory.
    pub fn new(fac: &'a mut Factory, verts: &'a [V]) -> Self {
        use cgmath::SquareMatrix;
        MeshBuilder {
            factory: fac,
            transform: Matrix4::identity(),
            vertices: verts,
        }
    }

    /// Sets the position of the mesh in 3D space.
    pub fn with_position<P: Into<Point3<f32>>>(mut self, pos: P) -> Self {
        use cgmath::EuclideanSpace;
        let trans = Matrix4::from_translation(pos.into().to_vec());
        self.transform.concat_self(&trans);
        self
    }

    /// Rotates the mesh a certain number of degrees around the given axis.
    pub fn with_rotation<Ax, An>(mut self, axis: Ax, angle: An) -> Self
        where Ax: Into<Vector3<f32>>,
              An: Into<Deg<f32>>
    {
        let rot = Matrix4::from_axis_angle(axis.into(), angle.into());
        self.transform.concat_self(&rot);
        self
    }

    /// Scales the mesh size according to the given value.
    pub fn with_scale(mut self, val: f32) -> Self {
        let scale = Matrix4::from_scale(val);
        self.transform.concat_self(&scale);
        self
    }

    /// Sets the transformation matrix of the mesh.
    ///
    /// This four-by-four matrix applies translation, rotation, and scaling to
    /// the mesh. It is often referred to in the computer graphics industry as
    /// the "model matrix".
    pub fn with_transform<M: Into<Matrix4<f32>>>(mut self, mat: M) -> Self {
        self.transform = mat.into();
        self
    }

    /// Builds and returns the new mesh.
    pub fn build(self) -> Mesh<V> {
        use gfx::traits::FactoryExt;

        let fac = self.factory;
        let verts = self.vertices;
        let (vbuf, slice) = fac.create_vertex_buffer_with_slice(verts, ());

        Mesh {
            slice: slice,
            transform: self.transform,
            vert_buf: vbuf,
        }
    }
}
