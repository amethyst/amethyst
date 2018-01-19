//!
//! Manage vertex and index buffers of single objects with ease.
//! 

use std::marker::PhantomData;
use std::mem::size_of;

use assets::{Asset, Handle, AssetStorage};
use core::cgmath::{Deg, Matrix4, Point3, SquareMatrix, Transform, Vector3};

use gfx_hal::{Backend, IndexCount, IndexType, Primitive, VertexCount};
use gfx_hal::buffer::{IndexBufferView, Usage};
use gfx_hal::command::RenderPassInlineEncoder;
use gfx_hal::memory::Properties;
use gfx_hal::pso::VertexBufferSet;

use smallvec::SmallVec;
use specs::{Component, DenseVecStorage};

use epoch::{CurrentEpoch, Eh, Epoch};
use formats::MeshData;
use hal::Hal;
use memory::{Allocator, Buffer, cast_vec};
use upload::{self, Uploader};
use utils::{is_slice_sorted, is_slice_sorted_by_key};
use vertex::{VertexFormat, VertexFormatSet, VertexFormatted};


/// Wraps container type (Like `Vec<V>`, `&[V]`, Box<[V]>, Cow<[V]> etc)
/// providing methods to build `VertexBuffer<B>` if `V` is `VertexFormatted`
/// or `IndexBuffer<B>` if `V` is `u16` or `u32`
pub struct Data<D, V> {
    data: D,
    pd: PhantomData<V>,
}

impl<D, V> Data<D, V>
where
    D: AsRef<[V]> + Into<Vec<V>>,
    V: VertexFormatted,
{
    fn new(data: D) -> Self {
        Data {
            data,
            pd: PhantomData,
        }
    }

    fn build_vertex<B>(
        self,
        allocator: &mut Allocator<B>,
        uploader: &mut Uploader<B>,
        current: &CurrentEpoch,
        device: &B::Device,
    ) -> Result<VertexBuffer<B>, ::failure::Error>
    where
        B: Backend,
    {
        let len = self.data.as_ref().len() as VertexCount;
        let mut buffer = allocator.create_buffer(
            device,
            len as _,
            Usage::VERTEX,
            Properties::DEVICE_LOCAL,
            false,
        )?;
        uploader.upload_buffer(allocator, current, device, &mut buffer, 0, self.data)?;
        Ok(VertexBuffer {
            buffer,
            format: V::VERTEX_FORMAT,
            len,
        })
    }
}

impl<D> Data<D, u16>
where
    D: AsRef<[u16]> + Into<Vec<u16>>,
{
    fn build_index<B>(
        self,
        allocator: &mut Allocator<B>,
        uploader: &mut Uploader<B>,
        current: &CurrentEpoch,
        device: &B::Device,
    ) -> Result<IndexBuffer<B>, ::failure::Error>
    where
        B: Backend,
    {
        let len = self.data.as_ref().len() as IndexCount;
        let mut buffer = allocator.create_buffer(
            device,
            len as _,
            Usage::INDEX,
            Properties::DEVICE_LOCAL,
            false,
        )?;
        uploader.upload_buffer(allocator, current, device, &mut buffer, 0, self.data)?;
        Ok(IndexBuffer {
            buffer,
            len,
            index_type: IndexType::U16,
        })
    }
}

impl<D> Data<D, u32>
where
    D: AsRef<[u32]> + Into<Vec<u32>>,
{
    fn build_index<B>(
        self,
        allocator: &mut Allocator<B>,
        uploader: &mut Uploader<B>,
        current: &CurrentEpoch,
        device: &B::Device,
    ) -> Result<IndexBuffer<B>, ::failure::Error>
    where
        B: Backend,
    {
        let len = self.data.as_ref().len() as IndexCount;

        let mut buffer = allocator.create_buffer(
            device,
            len as _,
            Usage::INDEX,
            Properties::DEVICE_LOCAL,
            false,
        )?;
        uploader.upload_buffer(allocator, current, device, &mut buffer, 0, self.data)?;

        Ok(IndexBuffer {
            buffer,
            len,
            index_type: IndexType::U32,
        })
    }
}

/// Heterogenous list of vertex data.
pub trait VertexDataList {
    /// Length of the list
    const LENGTH: usize;

    /// Build buffers from data.
    fn build<B>(
        self,
        allocator: &mut Allocator<B>,
        uploader: &mut Uploader<B>,
        current: &CurrentEpoch,
        device: &B::Device,
        output: &mut Vec<VertexBuffer<B>>,
    ) -> Result<(), ::failure::Error>
    where
        B: Backend;
}

/// Empty list implementation
impl VertexDataList for () {
    const LENGTH: usize = 0;
    fn build<B>(
        self,
        _allocator: &mut Allocator<B>,
        _uploader: &mut Uploader<B>,
        _current: &CurrentEpoch,
        _device: &B::Device,
        _output: &mut Vec<VertexBuffer<B>>,
    ) -> Result<(), ::failure::Error>
    where
        B: Backend,
    {
        Ok(())
    }
}


/// Non-empty list implementation
impl<D, V, L> VertexDataList for (Data<D, V>, L)
where
    D: AsRef<[V]> + Into<Vec<V>>,
    V: VertexFormatted,
    L: VertexDataList,
{
    const LENGTH: usize = 1 + L::LENGTH;
    fn build<B>(
        self,
        allocator: &mut Allocator<B>,
        uploader: &mut Uploader<B>,
        current: &CurrentEpoch,
        device: &B::Device,
        output: &mut Vec<VertexBuffer<B>>,
    ) -> Result<(), ::failure::Error>
    where
        B: Backend,
    {
        let (head, tail) = self;
        output.push(head.build_vertex(allocator, uploader, current, device)?);
        tail.build(allocator, uploader, current, device, output)
    }
}

/// Optional index data type.
pub trait IndexDataMaybe {
    /// Build buffer (or don't) from data.
    fn build<B>(
        self,
        allocator: &mut Allocator<B>,
        uploader: &mut Uploader<B>,
        current: &CurrentEpoch,
        device: &B::Device,
    ) -> Result<Option<IndexBuffer<B>>, ::failure::Error>
    where
        B: Backend;
}

/// None index data implementation
impl IndexDataMaybe for () {
    /// No data - no buffer
    fn build<B>(
        self,
        _allocator: &mut Allocator<B>,
        _uploader: &mut Uploader<B>,
        _current: &CurrentEpoch,
        _device: &B::Device,
    ) -> Result<Option<IndexBuffer<B>>, ::failure::Error>
    where
        B: Backend,
    {
        Ok(None)
    }
}

impl<D> IndexDataMaybe for Data<D, u16>
where
    D: AsRef<[u16]> + Into<Vec<u16>>,
{
    /// Build `u16` index buffer.
    fn build<B>(
        self,
        allocator: &mut Allocator<B>,
        uploader: &mut Uploader<B>,
        current: &CurrentEpoch,
        device: &B::Device,
    ) -> Result<Option<IndexBuffer<B>>, ::failure::Error>
    where
        B: Backend,
    {
        self.build_index(allocator, uploader, current, device)
            .map(Some)
    }
}

impl<D> IndexDataMaybe for Data<D, u32>
where
    D: AsRef<[u32]> + Into<Vec<u32>>,
{
    /// Build `u32` index buffer.
    fn build<B>(
        self,
        allocator: &mut Allocator<B>,
        uploader: &mut Uploader<B>,
        current: &CurrentEpoch,
        device: &B::Device,
    ) -> Result<Option<IndexBuffer<B>>, ::failure::Error>
    where
        B: Backend,
    {
        self.build_index(allocator, uploader, current, device)
            .map(Some)
    }
}

/// Vertex buffer with it's format
pub struct VertexBuffer<B: Backend> {
    buffer: Buffer<B>,
    format: VertexFormat<'static>,
    len: VertexCount,
}

/// Index buffer with it's type
pub struct IndexBuffer<B: Backend> {
    buffer: Buffer<B>,
    index_type: IndexType,
    len: IndexCount,
}

/// Mesh builder based on heterogenous lists.
/// It doesn't require data for buffers to be in `Vec`.
pub struct HMeshBuilder<V, I> {
    vertices: V,
    indices: I,
    prim: Primitive,
    transform: Matrix4<f32>,
}

impl HMeshBuilder<(), ()> {
    /// Create empty builder.
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
    /// Add indices buffer to the `HMeshBuilder`
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
        D: AsRef<[V]> + Into<Vec<V>>,
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
        use core::cgmath::EuclideanSpace;

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
    pub fn build<B>(self, hal: &mut Hal<B>) -> Result<Mesh<B>, ::failure::Error>
    where
        B: Backend,
    {
        Ok(Mesh {
            vbufs: {
                let mut vbufs = Vec::with_capacity(L::LENGTH);
                self.vertices.build(
                    &mut hal.allocator,
                    &mut hal.uploader,
                    &hal.current,
                    &hal.device,
                    &mut vbufs,
                )?;
                vbufs
            },
            ibuf: self.indices.build(
                &mut hal.allocator,
                &mut hal.uploader,
                &hal.current,
                &hal.device,
            )?,
            prim: self.prim,
            transform: self.transform,
        })
    }
}


/// Abstracts over two types of indices and their absence.
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

/// Generics-free mesh builder.
/// Useful for creating mesh from non-predefined set of data.
/// Like from glTF.
#[derive(Clone, Debug)]
pub struct MeshBuilder {
    vertices: SmallVec<[(Vec<u8>, VertexFormat<'static>); 16]>,
    indices: Option<(Vec<u8>, IndexType)>,
    prim: Primitive,
    transform: Matrix4<f32>,
}

impl MeshBuilder {
    /// Create empty builder.
    pub fn new() -> Self {
        MeshBuilder {
            vertices: SmallVec::new(),
            indices: None,
            prim: Primitive::TriangleList,
            transform: Matrix4::identity(),
        }
    }

    /// Add indices buffer to the `MeshBuilder`
    pub fn with_indices<I>(mut self, indices: I) -> Self
    where
        I: Into<Indices>,
    {
        self.indices = match indices.into() {
            Indices::None => None,
            Indices::U16(i) => Some((cast_vec(i), IndexType::U16)),
            Indices::U32(i) => Some((cast_vec(i), IndexType::U32)),
        };
        self
    }

    /// Add another vertices to the `MeshBuilder`
    pub fn with_vertices<V>(mut self, vertices: Vec<V>) -> Self
    where
        V: VertexFormatted,
    {
        self.vertices
            .push((cast_vec(vertices), V::VERTEX_FORMAT));
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
        use core::cgmath::EuclideanSpace;

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
    pub fn build<B>(
        self,
        allocator: &mut Allocator<B>,
        uploader: &mut Uploader<B>,
        current: &CurrentEpoch,
        device: &B::Device,
    ) -> Result<Mesh<B>, ::failure::Error>
    where
        B: Backend,
    {
        Ok(Mesh {
            vbufs: self.vertices
                .into_iter()
                .map(|(v, f)| {
                    let len = v.len() as VertexCount / f.stride as VertexCount;
                    Ok(VertexBuffer {
                        buffer: {
                            let mut buffer = allocator.create_buffer(
                                device,
                                v.len() as _,
                                Usage::VERTEX,
                                Properties::DEVICE_LOCAL,
                                false,
                            )?;
                            uploader.upload_buffer(allocator, current, device, &mut buffer, 0, v)?;
                            buffer
                        },
                        format: f,
                        len,
                    })
                })
                .collect::<Result<_, ::failure::Error>>()?,
            ibuf: match self.indices {
                None => None,
                Some((i, t)) => {
                    let stride = match t {
                        IndexType::U16 => size_of::<u16>(),
                        IndexType::U32 => size_of::<u32>(),
                    };
                    let len = i.len() as IndexCount / stride as IndexCount;
                    Some(IndexBuffer {
                        buffer: {
                            let mut buffer = allocator.create_buffer(
                                device,
                                i.len() as _,
                                Usage::INDEX,
                                Properties::DEVICE_LOCAL,
                                false,
                            )?;
                            uploader.upload_buffer(allocator, current, device, &mut buffer, 0, i)?;
                            buffer
                        },
                        index_type: t,
                        len,
                    })
                }
            },
            prim: self.prim,
            transform: self.transform,
        })
    }
}

/// Single mesh is a collection of buffers that provides available attributes.
/// Exactly one mesh is used per drawing call in common.
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
    /// Build new mesh with `HMeshBuilder`
    pub fn new() -> HMeshBuilder<(), ()> {
        HMeshBuilder::new()
    }

    /// Primitive type of the `Mesh`
    pub fn primitive(&self) -> Primitive {
        self.prim
    }

    /// Bind buffers to specified attribute locations.
    pub fn bind<'a>(
        &'a self,
        until: Epoch,
        format_set: VertexFormatSet<'static>,
        vertex: &mut VertexBufferSet<'a, B>,
    ) -> Result<Bind<B>, BindError> {
        debug_assert!(is_slice_sorted(format_set));
        debug_assert!(is_slice_sorted_by_key(
            &self.vbufs,
            |vbuf| vbuf.format.attributes,
        ));
        debug_assert!(vertex.0.is_empty());

        let mut last = 0;
        let mut vertex_count = None;
        for format in format_set {
            if let Some(index) = find_compatible_buffer(&self.vbufs[last..], format) {
                // Ensure buffer is valid
                Eh::make_valid_until(&self.vbufs[index].buffer, until);
                vertex.0.push((self.vbufs[index].buffer.raw(), 0));
                last = index;
                assert!(vertex_count.is_none() || vertex_count == Some(self.vbufs[index].len));
                vertex_count = Some(self.vbufs[index].len);
            } else {
                // Can't bind
                return Err(BindError::Incompatible {
                    format: format_set,
                });
            }
        }
        Ok(
            self.ibuf
                .as_ref()
                .map(|ibuf| {
                    Eh::make_valid_until(&ibuf.buffer, until);
                    Bind::Indexed {
                        index: IndexBufferView {
                            buffer: ibuf.buffer.raw(),
                            offset: 0,
                            index_type: ibuf.index_type,
                        },
                        count: ibuf.len,
                    }
                })
                .unwrap_or(Bind::Unindexed {
                    count: vertex_count.unwrap_or(0),
                }),
        )
    }

    pub fn dispose(self, allocator: &mut Allocator<B>) {
        if let Some(ibuf) = self.ibuf {
            allocator.destroy_buffer(ibuf.buffer);
        }

        for vbuf in self.vbufs {
            allocator.destroy_buffer(vbuf.buffer);
        }
    }

    pub fn transform(&self) -> &Matrix4<f32> {
        &self.transform
    }
}

impl<B> Component for Mesh<B>
where
    B: Backend,
{
    type Storage = DenseVecStorage<Self>;
}


/// A handle to a mesh.
pub type MeshHandle<B: Backend> = Handle<Mesh<B>>;

/// A storage of the meshes.
pub type MeshStorage<B: Backend> = AssetStorage<Mesh<B>>;

impl<B> Asset for Mesh<B>
where
    B: Backend,
{
    type Data = MeshData;
    type HandleStorage = DenseVecStorage<MeshHandle<B>>;
}


#[derive(Fail, Debug)]
pub enum BindError {
    #[fail(display = "Mesh is incompatible with format: {:?}", format)]
    Incompatible {
        format: VertexFormatSet<'static>,
    },
}


/// Result of buffers bindings.
/// It only contains `IndexBufferView` (if index buffers exists)
/// and vertex count.
/// Vertex buffers are in separate `VertexBufferSet`
pub enum Bind<'a, B: Backend> {
    Indexed {
        index: IndexBufferView<'a, B>,
        count: IndexCount,
    },
    Unindexed {
        count: VertexCount,
    },
}

impl<'a, B> Bind<'a, B>
where
    B: Backend,
{
    /// Record drawing command for this biding.
    pub fn draw_inline(self, vertex: VertexBufferSet<B>, encoder: &mut RenderPassInlineEncoder<B>) {
        encoder.bind_vertex_buffers(vertex);
        match self {
            Bind::Indexed { index, count } => {
                encoder.bind_index_buffer(index);
                encoder.draw_indexed(0..count, 0, 0..1);
            }
            Bind::Unindexed { count } => {
                encoder.draw(0..count, 0..1);
            }
        }
    }
}


/// Helper function to find buffer with compatible format.
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


/// Check is vertex format `left` is compatible with `right`.
/// `left` must have same `stride` and contain all attributes from `right`.
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
