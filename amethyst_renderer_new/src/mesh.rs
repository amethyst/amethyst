use std::marker::PhantomData;
use std::mem::size_of;

use cgmath::{Deg, Matrix4, Point3, SquareMatrix, Transform, Vector3};

use gfx_hal::{Backend, Device, IndexType, Primitive};
use gfx_hal::buffer::Usage;
use gfx_hal::memory::{Pod, cast_slice};
use gfx_hal::pso::{ElemStride, VertexBufferSet};

use smallvec::SmallVec;

use memory::{self, Allocator, cast_pod_vec};
use utils::{is_slice_sorted, is_slice_sorted_by_key};
use vertex::{Attributes, VertexFormat, VertexFormatSet, VertexFormatted};

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
    V: VertexFormatted,
{
    pub fn new(data: D) -> Self {
        Data {
            data,
            pd: PhantomData,
        }
    }

    pub fn size(&self) -> usize {
        self.data.as_ref().len() * V::VERTEX_FORMAT.stride as usize
    }

    pub fn stride(&self) -> ElemStride {
        V::VERTEX_FORMAT.stride
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
        Ok(VertexBuffer {
            buffer: make_buffer(
                cast_slice(self.data.as_ref()),
                V::VERTEX_FORMAT.stride,
                allocator,
                device,
            )?,
            format: V::VERTEX_FORMAT,
            len: self.data.as_ref().len(),
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
    V: VertexFormatted,
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
        Ok(Some(IndexBuffer {
            buffer: make_buffer(
                cast_slice(self.data.as_ref()),
                size_of::<u16>() as _,
                allocator,
                device,
            )?,
            len: self.data.as_ref().len(),
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
        Ok(Some(IndexBuffer {
            buffer: make_buffer(
                cast_slice(self.data.as_ref()),
                size_of::<u32>() as _,
                allocator,
                device,
            )?,
            len: self.data.as_ref().len(),
            index_type: IndexType::U32,
        }))
    }
}

pub struct VertexBuffer<B: Backend> {
    buffer: B::Buffer,
    format: VertexFormat<'static>,
    len: usize,
}

pub struct IndexBuffer<B: Backend> {
    buffer: B::Buffer,
    index_type: IndexType,
    len: usize,
}

pub struct HMeshBuilder<V, I> {
    vertices: V,
    indices: I,
    prim: Primitive,
    transform: Matrix4<f32>,
}

impl HMeshBuilder<(), ()> {
    fn new() -> Self {
        HMeshBuilder {
            vertices: (),
            indices: (),
            prim: Primitive::TriangleList,
            transform: Matrix4::identity(),
        }
    }
}

impl<L> HMeshBuilder<L, ()>
where
    L: VertexDataList,
{
    /// Add indices buffer to the `MeshBuiler`
    pub fn with_indices<I>(self, indices: I) -> HMeshBuilder<L, I>
    where
        I: IndexDataMaybe,
    {
        HMeshBuilder {
            vertices: self.vertices,
            indices: indices,
            prim: self.prim,
            transform: self.transform,
        }
    }
}

impl<L, I> HMeshBuilder<L, I>
where
    L: VertexDataList,
    I: IndexDataMaybe,
{
    /// Add another vertices to the `HMeshBuilder`
    pub fn with_vertices<D, V>(self, vertices: D) -> HMeshBuilder<(Data<D, V>, L), I>
    where
        D: AsRef<[V]>,
        V: VertexFormatted,
    {
        HMeshBuilder {
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

impl<L, I> HMeshBuilder<L, I>
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

pub enum Indices {
    None,
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl From<Vec<u16>> for Indices {
    fn from(vec: Vec<u16>) -> Indices {
        Indices::U16(vec)
    }
}

impl From<Vec<u32>> for Indices {
    fn from(vec: Vec<u32>) -> Indices {
        Indices::U32(vec)
    }
}

pub struct MeshBuilder {
    vertices: SmallVec<[(Vec<u8>, VertexFormat<'static>); 16]>,
    indices: Option<(Vec<u8>, IndexType)>,
    prim: Primitive,
    transform: Matrix4<f32>,
}

impl MeshBuilder {
    fn new() -> Self {
        MeshBuilder {
            vertices: SmallVec::new(),
            indices: None,
            prim: Primitive::TriangleList,
            transform: Matrix4::identity(),
        }
    }

    /// Add indices buffer to the `MeshBuiler`
    pub fn with_indices<I>(mut self, indices: I) -> Self
    where
        I: Into<Indices>,
    {
        self.indices = match indices.into() {
            Indices::None => None,
            Indices::U16(i) => Some((cast_pod_vec(i), IndexType::U16)),
            Indices::U32(i) => Some((cast_pod_vec(i), IndexType::U32)),
        };
        self
    }

    /// Add another vertices to the `MeshBuilder`
    pub fn with_vertices<V>(mut self, vertices: Vec<V>) -> Self
    where
        V: VertexFormatted,
    {
        self.vertices.push(
            (cast_pod_vec(vertices), V::VERTEX_FORMAT),
        );
        self
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

    /// Builds and returns the new mesh.
    pub fn build<A, B>(self, allocator: &mut A, device: &mut B::Device) -> Result<Mesh<B>>
    where
        A: Allocator<B>,
        B: Backend,
    {
        Ok(Mesh {
            vbufs: self.vertices
                .into_iter()
                .map(|(v, f)| {
                    Ok(VertexBuffer {
                        buffer: make_buffer(&v, f.stride, allocator, device)?,
                        format: f,
                        len: v.len() / f.stride as usize,
                    })
                })
                .collect::<Result<_>>()?,
            ibuf: match self.indices {
                None => None,
                Some((i, t)) => {
                    let stride = match t {
                        IndexType::U16 => size_of::<u16>(),
                        IndexType::U32 => size_of::<u32>(),
                    };
                    Some(IndexBuffer {
                        buffer: make_buffer(&i, stride as _, allocator, device)?,
                        index_type: t,
                        len: i.len() / stride,
                    })
                }
            },
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
    /// Builde new mesh with `HMeshBuilder`
    pub fn new() -> HMeshBuilder<(), ()> {
        HMeshBuilder::new()
    }

    pub fn bind<'a>(
        &'a self,
        format_set: VertexFormatSet,
        output: &mut VertexBufferSet<'a, B>,
    ) -> Result<()> {
        debug_assert!(is_slice_sorted(format_set));
        debug_assert!(is_slice_sorted_by_key(
            &self.vbufs,
            |vbuf| vbuf.format.attributes,
        ));
        debug_assert!(output.0.is_empty());

        let mut last = 0;
        for format in format_set {
            if let Some(index) = find_compatible_buffer(&self.vbufs[last..], format) {
                output.0.push((&self.vbufs[index].buffer, 0));
                last = index;
            } else {
                // Can't bind
                return Err(ErrorKind::Incompatible.into());
            }
        }
        Ok(())
    }

    fn transformt(&self) -> &Matrix4<f32> {
        &self.transform
    }
}

fn find_compatible_buffer<B>(vbufs: &[VertexBuffer<B>], format: &VertexFormat) -> Option<usize>
where
    B: Backend,
{
    debug_assert!(is_slice_sorted(format.attributes));
    for (i, vbuf) in vbufs.iter().enumerate() {
        if is_compatible(&vbuf.format, format) {
            return Some(i);
        }
    }
    None
}

fn is_compatible(left: &VertexFormat, right: &VertexFormat) -> bool {
    if left.stride != right.stride {
        return false;
    }

    let mut i = 0;
    right.attributes.iter().all(|r| {
        left.attributes
            .iter()
            .skip(i)
            .position(|l| *l == *r)
            .map_or(false, |p| {
                i = p;
                true
            })
    })
}


fn make_buffer<A, B>(
    data: &[u8],
    stride: ElemStride,
    allocator: &mut A,
    device: &mut B::Device,
) -> Result<B::Buffer>
where
    A: Allocator<B>,
    B: Backend,
{
    let size = data.len();
    let buffer = allocator.allocate_buffer(
        device,
        size,
        stride as _,
        Usage::VERTEX,
    )?;
    {
        // Copy vertex data
        let mut writer = device
            .acquire_mapping_writer::<u8>(&buffer, 0..size as u64)
            .map_err(memory::Error::from)?;
        writer.copy_from_slice(data);
    }

    Ok(buffer)
}
