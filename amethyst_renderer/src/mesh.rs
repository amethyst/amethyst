//! Mesh resource.

use cgmath::{Deg, Matrix4, Point3, Transform, Vector3};
use error::Result;
use gfx::Primitive;
use std::marker::PhantomData;
use types::{Factory, RawBuffer, Slice};
use vertex::{Attribute, VertexFormat};

/// Represents a polygonal mesh.
#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    attrs: Vec<Attribute>,
    prim: Primitive,
    slice: Slice,
    transform: Matrix4<f32>,
    vbuf: RawBuffer,
}

impl Mesh {
    /// Builds a new mesh from the given vertices.
    pub fn build<D, V>(verts: D) -> MeshBuilder<D, V>
        where D: AsRef<[V]>,
              V: VertexFormat,
    {
        MeshBuilder::new(verts)
    }

    /// Returns a list of all vertex attributes needed by this mesh.
    pub fn attributes(&self) -> &[Attribute] {
        self.attrs.as_ref()
    }

    /// Returns the mesh's vertex buffer and associated buffer slice.
    pub fn geometry(&self) -> (&RawBuffer, &Slice) {
        (&self.vbuf, &self.slice)
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
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MeshBuilder<D, V> {
    prim: Primitive,
    transform: Matrix4<f32>,
    vertices: D,
    pd: PhantomData<V>,
}

impl<D, V> MeshBuilder<D, V>
    where D: AsRef<[V]>,
          V: VertexFormat,
{
    /// Creates a new `MeshBuilder` with the given vertices.
    pub fn new(verts: D) -> Self {
        use cgmath::SquareMatrix;

        MeshBuilder {
            prim: Primitive::TriangleList,
            transform: Matrix4::identity(),
            vertices: verts,
            pd: PhantomData,
        }
    }

    /// Sets the primitive type of the mesh.
    ///
    /// By default, meshes are constructed as triangle lists.
    pub fn with_prim_type(mut self, prim: Primitive) -> Self {
        self.prim = prim;
        self
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
    pub(crate) fn build(self, fac: &mut Factory) -> Result<Mesh> {
        use gfx::{Bind, Factory, IndexBuffer};
        use gfx::buffer::Role;
        use gfx::memory::cast_slice;

        let verts = cast_slice(self.vertices.as_ref());
        let count = self.vertices.as_ref().len();
        let stride = verts.len() / count;
        let role = Role::Vertex;
        let bind = Bind::empty();

        let vbuf = fac.create_buffer_immutable_raw(verts, stride, role, bind)?;
        let slice = Slice {
            start: 0,
            end: count as u32,
            base_vertex: 0,
            instances: None,
            buffer: IndexBuffer::Auto,
        };

        Ok(Mesh {
            attrs: V::attributes().as_ref().to_vec(),
            prim: self.prim,
            slice: slice,
            transform: self.transform,
            vbuf: vbuf,
        })
    }
}

