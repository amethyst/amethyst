use std::marker::PhantomData;

use cgmath::{Deg, Matrix4, Point3, SquareMatrix, Transform, Vector3};

use gfx_hal::Backend;
use gfx_hal::buffer::Usage;
use gfx_hal::{Device as GfxDevice, IndexType, Primitive};
use gfx_hal::pso::VertexBufferSet;

use memory::{self, Allocator};
use utils::{is_slice_sorted, is_slice_sorted_by_key};
use vertex::{Attributes, AttributesSet, VertexFormat};

error_chain! {
    links {
        Memory(memory::Error, memory::ErrorKind);
    }

    errors {
        Incompatible {
            description("Incompatible"),
            display("Incompatible"),
        }
    }
}

pub struct Data<D, V> {
    data: D,
    pd: PhantomData<(V)>,
}

impl<D, V> Data<D, V>
where
    D: AsRef<[V]>,
    V: VertexFormat,
{
    pub fn new(data: D) -> Self {
        Data {
            data,
            pd: PhantomData,
        }
    }

    pub fn size(&self) -> usize {
        self.data.as_ref().len() * V::size()
    }

    pub fn stride(&self) -> usize {
        V::size()
    }

    pub(crate) fn build<A, B>(
        self,
        allocator: &mut A,
        device: &mut B::Device,
    ) -> Result<VertexBuffer<B>>
    where
        A: Allocator<B>,
        B: Backend,
    {
        // Allocate buffer
        let buffer = allocator.allocate_buffer(device, self.size(), self.stride(), Usage::VERTEX)?;
        {
            // Copy vertex data
            let mut writer = device
                .acquire_mapping_writer::<V>(&buffer, 0..self.size() as u64)
                .map_err(memory::Error::from)?;
            writer.copy_from_slice(self.data.as_ref());
        }

        Ok(VertexBuffer {
            buffer,
            size: self.size(),
            attributes: V::ATTRIBUTES,
        })
    }
}


/// List of vertex data
pub trait VertexDataList {
    const LENGTH: usize;
    fn build<A, B>(
        self,
        allocator: &mut A,
        device: &mut B::Device,
        output: &mut Vec<VertexBuffer<B>>,
    ) -> Result<()>
    where
        A: Allocator<B>,
        B: Backend;
}

impl VertexDataList for () {
    const LENGTH: usize = 0;
    fn build<A, B>(
        self,
        allocator: &mut A,
        device: &mut B::Device,
        output: &mut Vec<VertexBuffer<B>>,
    ) -> Result<()>
    where
        A: Allocator<B>,
        B: Backend,
    {
        Ok(())
    }
}

impl<D, V, L> VertexDataList for (Data<D, V>, L)
where
    D: AsRef<[V]>,
    V: VertexFormat,
    L: VertexDataList,
{
    const LENGTH: usize = 1 + L::LENGTH;
    fn build<A, B>(
        self,
        allocator: &mut A,
        device: &mut B::Device,
        output: &mut Vec<VertexBuffer<B>>,
    ) -> Result<()>
    where
        A: Allocator<B>,
        B: Backend,
    {
        let (head, tail) = self;
        output.push(head.build(allocator, device)?);
        tail.build(allocator, device, output)
    }
}

pub trait IndexDataMaybe {
    fn build<A, B>(
        self,
        allocator: &mut A,
        device: &mut B::Device,
    ) -> Result<Option<IndexBuffer<B>>>
    where
        A: Allocator<B>,
        B: Backend;
}

impl IndexDataMaybe for () {
    fn build<A, B>(
        self,
        allocator: &mut A,
        device: &mut B::Device,
    ) -> Result<Option<IndexBuffer<B>>>
    where
        B: Backend,
    {
        Ok(None)
    }
}

impl<D> IndexDataMaybe for Data<D, u16>
where
    D: AsRef<[u16]>,
{
    fn build<A, B>(
        self,
        allocator: &mut A,
        device: &mut B::Device,
    ) -> Result<Option<IndexBuffer<B>>>
    where
        A: Allocator<B>,
        B: Backend,
    {
        let data = self.data.as_ref();
        let stride = 2;
        let size = data.len() * stride;
        // Allocate buffer
        let buffer = allocator.allocate_buffer(device, size, stride, Usage::VERTEX)?;
        {
            // Copy vertex data
            let mut writer = device
                .acquire_mapping_writer::<u16>(&buffer, 0..size as u64)
                .map_err(memory::Error::from)?;
            writer.copy_from_slice(data);
        }

        Ok(Some(IndexBuffer {
            buffer,
            size: size,
            index_type: IndexType::U16,
        }))
    }
}

impl<D> IndexDataMaybe for Data<D, u32>
where
    D: AsRef<[u32]>,
{
    fn build<A, B>(
        self,
        allocator: &mut A,
        device: &mut B::Device,
    ) -> Result<Option<IndexBuffer<B>>>
    where
        A: Allocator<B>,
        B: Backend,
    {
        let data = self.data.as_ref();
        let stride = 4;
        let size = data.len() * stride;
        // Allocate buffer
        let buffer = allocator.allocate_buffer(device, size, stride, Usage::VERTEX)?;
        {
            // Copy vertex data
            let mut writer = device
                .acquire_mapping_writer::<u32>(&buffer, 0..size as u64)
                .map_err(memory::Error::from)?;
            writer.copy_from_slice(data);
        }

        Ok(Some(IndexBuffer {
            buffer,
            size: size,
            index_type: IndexType::U32,
        }))
    }
}

pub struct VertexBuffer<B: Backend> {
    buffer: B::Buffer,
    size: usize,
    attributes: Attributes<'static>,
}

pub struct IndexBuffer<B: Backend> {
    buffer: B::Buffer,
    size: usize,
    index_type: IndexType,
}

pub struct MeshBuilder<V, I> {
    vertices: V,
    indices: I,
    prim: Primitive,
    transform: Matrix4<f32>,
}

impl MeshBuilder<(), ()> {
    fn new() -> Self {
        MeshBuilder {
            vertices: (),
            indices: (),
            prim: Primitive::TriangleList,
            transform: Matrix4::identity(),
        }
    }
}

impl<L> MeshBuilder<L, ()>
where
    L: VertexDataList,
{
    /// Add indices buffer to the `MeshBuiler`
    pub fn with_indices<I>(self, indices: I) -> MeshBuilder<L, I>
    where
        I: IndexDataMaybe,
    {
        MeshBuilder {
            vertices: self.vertices,
            indices: indices,
            prim: self.prim,
            transform: self.transform,
        }
    }
}

impl<L, I> MeshBuilder<L, I>
where
    L: VertexDataList,
    I: IndexDataMaybe,
{
    /// Add another vertices to the `MeshBuilder`
    pub fn with_vertices<D, V>(self, vertices: D) -> MeshBuilder<(Data<D, V>, L), I>
    where
        D: AsRef<[V]>,
        V: VertexFormat,
    {
        MeshBuilder {
            vertices: (Data::new(vertices), self.vertices),
            indices: self.indices,
            prim: self.prim,
            transform: self.transform,
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
}

impl<L, I> MeshBuilder<L, I>
where
    L: VertexDataList,
    I: IndexDataMaybe,
{
    /// Builds and returns the new mesh.
    pub fn build<A, B>(self, allocator: &mut A, device: &mut B::Device) -> Result<Mesh<B>>
    where
        A: Allocator<B>,
        B: Backend,
    {
        Ok(Mesh {
            vbufs: {
                let mut vbufs = Vec::with_capacity(L::LENGTH);
                self.vertices.build(allocator, device, &mut vbufs)?;
                vbufs
            },
            ibuf: self.indices.build(allocator, device)?,
            prim: self.prim,
            transform: self.transform,
        })
    }
}

pub struct Mesh<B: Backend> {
    vbufs: Vec<VertexBuffer<B>>,
    ibuf: Option<IndexBuffer<B>>,
    prim: Primitive,
    transform: Matrix4<f32>,
}

impl<B> Mesh<B>
where
    B: Backend,
{
    /// Builde new mesh with `MeshBuilder`
    pub fn new() -> MeshBuilder<(), ()> {
        MeshBuilder::new()
    }

    pub fn bind<'a>(
        &'a self,
        attributes_set: AttributesSet,
        output: &mut VertexBufferSet<'a, B>,
    ) -> Result<()> {
        debug_assert!(is_slice_sorted(attributes_set));
        debug_assert!(is_slice_sorted_by_key(&self.vbufs, |vbuf| vbuf.attributes));
        debug_assert!(output.0.is_empty());

        let mut last = 0;
        for attributes in attributes_set {
            if let Some(index) = find_compatible_buffer(&self.vbufs[last..], attributes) {
                output.0.push((&self.vbufs[index].buffer, 0));
                last = index;
            } else {
                // Can't bind
                return Err(ErrorKind::Incompatible.into());
            }
        }
        Ok(())
    }
}

fn find_compatible_buffer<B>(vbufs: &[VertexBuffer<B>], attributes: Attributes) -> Option<usize>
where
    B: Backend,
{
    debug_assert!(is_slice_sorted(attributes));
    for i in 0..vbufs.len() {
        let mut find = attributes.iter();
        let mut next = find.next();
        let mut j = 0;
        while let Some(attr) = next {
            if j == vbufs[i].attributes.len() {
                // try next vbuf
                break;
            } else if *attr == vbufs[i].attributes[j] {
                // match. search next attribute
                next = find.next();
                j += 1;
            } else {
                // continue searching
                j += 1;
            }
        }
        return Some(i);
    }
    None
}
