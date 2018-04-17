//! Mesh resource.

use std::iter::{once, Chain, Once};
use std::marker::PhantomData;

use amethyst_assets::Handle;

use amethyst_core::cgmath::{Deg, Matrix4, Point3, Transform, Vector3};
use gfx::Primitive;

use error::Result;
use types::{Factory, RawBuffer, Slice};
use vertex::{Attributes, VertexFormat};

/// Raw buffer with its attributes
#[derive(Clone, Debug)]
pub struct VertexBuffer {
    attrs: Attributes<'static>,
    raw: RawBuffer,
}

/// Vertex data that can be built into `VertexBuffer`
#[doc(hidden)]
pub trait VertexData {
    const ATTRIBUTES: Attributes<'static>;

    /// Get vertex count in buffer
    fn len(&self) -> usize;

    /// Build `VertexBuffer`
    fn build(&self, factory: &mut Factory) -> Result<VertexBuffer>;
}

/// Construct new vertex data from raw data and vertex format
pub fn vertex_data<D, V>(data: D) -> (D, PhantomData<V>)
where
    D: AsRef<[V]>,
    V: VertexFormat,
{
    (data, PhantomData)
}

impl<D, V> VertexData for (D, PhantomData<V>)
where
    D: AsRef<[V]>,
    V: VertexFormat,
{
    const ATTRIBUTES: Attributes<'static> = V::ATTRIBUTES;

    fn len(&self) -> usize {
        self.0.as_ref().len()
    }

    fn build(&self, factory: &mut Factory) -> Result<VertexBuffer> {
        use gfx::buffer::Role;
        use gfx::memory::{cast_slice, Bind};
        use gfx::Factory;

        let verts = self.0.as_ref();
        let slice = cast_slice(verts);
        let stride = slice.len() / verts.len();
        let role = Role::Vertex;
        let bind = Bind::empty();

        let vbuf = factory.create_buffer_immutable_raw(slice, stride, role, bind)?;
        Ok(VertexBuffer {
            attrs: V::ATTRIBUTES,
            raw: vbuf,
        })
    }
}

/// Set of vertex data
#[doc(hidden)]
pub trait VertexDataSet {
    /// Iterator for `VertexBuffer`s built
    type VertexBufferIter: Iterator<Item = VertexBuffer>;

    /// Get smalles vertex count across buffers
    fn len(&self) -> usize;

    /// Build `VertexBuffer`s
    fn build(&self, factory: &mut Factory) -> Result<Self::VertexBufferIter>;
}

impl<H> VertexDataSet for (H, ())
where
    H: VertexData,
{
    type VertexBufferIter = Once<VertexBuffer>;

    fn len(&self) -> usize {
        self.0.len()
    }

    fn build(&self, factory: &mut Factory) -> Result<Self::VertexBufferIter> {
        let (ref head, _) = *self;
        Ok(once(head.build(factory)?))
    }
}

impl<H, T> VertexDataSet for (H, T)
where
    H: VertexData,
    T: VertexDataSet,
{
    type VertexBufferIter = Chain<Once<VertexBuffer>, T::VertexBufferIter>;

    fn len(&self) -> usize {
        use std::cmp::min;
        min(self.0.len(), self.1.len())
    }

    fn build(&self, factory: &mut Factory) -> Result<Self::VertexBufferIter> {
        let (ref head, ref tail) = *self;
        Ok(once(head.build(factory)?).chain(tail.build(factory)?))
    }
}

/// A handle to a mesh.
pub type MeshHandle = Handle<Mesh>;

/// Represents a polygonal mesh.
#[derive(Clone, Debug)]
pub struct Mesh {
    slice: Slice,
    transform: Matrix4<f32>,
    vbufs: Vec<VertexBuffer>,
}

impl Mesh {
    /// Builds a new mesh from the given vertices.
    pub fn build<D, V>(verts: D) -> MeshBuilder<((D, PhantomData<V>), ())>
    where
        D: AsRef<[V]>,
        V: VertexFormat,
    {
        MeshBuilder::new(verts)
    }

    /// Returns the mesh's vertex buffer which matches requested attributes
    pub fn buffer(&self, attributes: Attributes) -> Option<&RawBuffer> {
        for vbuf in self.vbufs.iter() {
            let mut find = attributes.iter();
            let mut next = find.next();
            let mut i = 0;
            let mut j = 0;
            loop {
                let attrs = vbuf.attrs;
                match next {
                    Some(&attr) => {
                        if i == attrs.len() {
                            // try next vbuf
                            break;
                        } else if attrs[(i + j) % attrs.len()] == attr {
                            // match. search next attribute
                            next = find.next();
                            j = i;
                            i = 0;
                        } else {
                            // continue searching
                            i += 1;
                        }
                    }
                    None => {
                        // All atributes found
                        return Some(&vbuf.raw);
                    }
                }
            }
        }

        // None of the vertex buffers match requested attributes
        None
    }

    /// Returns associated `Slice`
    pub fn slice(&self) -> &Slice {
        &self.slice
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
pub struct MeshBuilder<T> {
    prim: Primitive,
    transform: Matrix4<f32>,
    vertices: T,
}

impl<D, V> MeshBuilder<((D, PhantomData<V>), ())>
where
    D: AsRef<[V]>,
    V: VertexFormat,
{
    /// Creates a new `MeshBuilder` with the given vertices.
    pub fn new(verts: D) -> Self {
        use amethyst_core::cgmath::SquareMatrix;
        assert!(check_attributes_are_sorted(V::ATTRIBUTES));
        MeshBuilder {
            prim: Primitive::TriangleList,
            transform: Matrix4::identity(),
            vertices: (vertex_data(verts), ()),
        }
    }
}

impl<T> MeshBuilder<T>
where
    T: VertexDataSet,
{
    /// Add another vertices to the `MeshBuilder`
    pub fn with_buffer<D, V>(self, verts: D) -> MeshBuilder<((D, PhantomData<V>), T)>
    where
        D: AsRef<[V]>,
        V: VertexFormat,
    {
        assert!(check_attributes_are_sorted(V::ATTRIBUTES));
        MeshBuilder {
            prim: self.prim,
            transform: self.transform,
            vertices: (vertex_data(verts), self.vertices),
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
        use amethyst_core::cgmath::EuclideanSpace;

        let trans = Matrix4::from_translation(pos.into().to_vec());
        self.transform.concat_self(&trans);
        self
    }

    /// Rotates the mesh a certain number of degrees around the given axis.
    pub fn with_rotation<Ax, An>(mut self, axis: Ax, angle: An) -> Self
    where
        Ax: Into<Vector3<f32>>,
        An: Into<Deg<f32>>,
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
    pub fn build(self, fac: &mut Factory) -> Result<Mesh> {
        use gfx::IndexBuffer;
        let count = self.vertices.len();

        let slice = Slice {
            start: 0,
            end: count as u32,
            base_vertex: 0,
            instances: None,
            buffer: IndexBuffer::Auto,
        };

        Ok(Mesh {
            slice: slice,
            transform: self.transform,
            vbufs: self.vertices.build(fac)?.collect(),
        })
    }
}

/// Check that attributes are sorted
fn check_attributes_are_sorted(attrs: Attributes) -> bool {
    let mut last = 0;
    for attr in attrs {
        if last > attr.1.offset {
            return false;
        }
        last = attr.1.offset;
    }
    true
}
