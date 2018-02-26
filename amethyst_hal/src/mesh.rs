//!
//! Manage vertex and index buffers of single objects with ease.
//!

use std::borrow::Cow;
use std::marker::PhantomData;
use std::mem::size_of;

use core::cgmath::{Deg, Matrix4, Point3, SquareMatrix, Transform, Vector3};

use hal::{Backend, IndexCount, IndexType, Primitive, VertexCount};
use hal::buffer::{IndexBufferView, Usage};
use hal::command::{RenderSubpassCommon};
use hal::memory::{Properties, cast_slice};
use hal::pso::VertexBufferSet;

use smallvec::SmallVec;

use {Buffer, Error};
use factory::Factory;
use utils::{is_slice_sorted, is_slice_sorted_by_key, cast_cow};
use vertex::{VertexFormat, AsVertexFormat};

// /// Wraps container type (Like `Vec<V>`, `&[V]`, Box<[V]>, Cow<[V]> etc)
// /// providing methods to build `VertexBuffer<B>` if `V` is `AsVertexFormat`
// /// or `IndexBuffer<B>` if `V` is `u16` or `u32`
// pub struct Data<D, V> {
//     data: D,
//     pd: PhantomData<V>,
// }

// impl<D, V> Data<D, V>
// where
//     D: AsRef<[V]>,
//     V: AsVertexFormat,
// {
//     fn new(data: D) -> Self {
//         Data {
//             data,
//             pd: PhantomData,
//         }
//     }

//     fn build_vertex<B>(
//         self,
//         factory: &mut Factory<B>,
//     ) -> Result<VertexBuffer<B>, Error>
//     where
//         B: Backend,
//     {
//         let len = self.data.as_ref().len() as VertexCount;
//         let mut buffer = factory.create_buffer(
//             Properties::DEVICE_LOCAL,
//             len as _,
//             Usage::VERTEX | Usage::TRANSFER_DST,
//         )?;
//         factory.upload_buffer(&mut buffer, 0, cast_slice(self.data.as_ref()))?;
//         Ok(VertexBuffer {
//             buffer,
//             format: V::VERTEX_FORMAT,
//             len,
//         })
//     }
// }

// impl<D> Data<D, u16>
// where
//     D: AsRef<[u16]>,
// {
//     fn build_index<B>(
//         self,
//         factory: &mut Factory<B>,
//     ) -> Result<IndexBuffer<B>, Error>
//     where
//         B: Backend,
//     {
//         let len = self.data.as_ref().len() as IndexCount;
//         let mut buffer = factory.create_buffer(
//             Properties::DEVICE_LOCAL,
//             (len * 2) as _,
//             Usage::INDEX | Usage::TRANSFER_DST,
//         )?;
//         factory.upload_buffer(&mut buffer, 0, cast_slice(self.data.as_ref()))?;
//         Ok(IndexBuffer {
//             buffer,
//             index_type: IndexType::U16,
//             len,
//         })
//     }
// }

// impl<D> Data<D, u32>
// where
//     D: AsRef<[u32]>,
// {
//     fn build_index<B>(
//         self,
//         factory: &mut Factory<B>,
//     ) -> Result<IndexBuffer<B>, Error>
//     where
//         B: Backend,
//     {
//         let len = self.data.as_ref().len() as IndexCount;
//         let mut buffer = factory.create_buffer(
//             Properties::DEVICE_LOCAL,
//             (len * 4) as _,
//             Usage::INDEX | Usage::TRANSFER_DST,
//         )?;
//         factory.upload_buffer(&mut buffer, 0, cast_slice(self.data.as_ref()))?;
//         Ok(IndexBuffer {
//             buffer,
//             index_type: IndexType::U32,
//             len,
//         })
//     }
// }

// /// Heterogenous list of vertex data.
// pub trait VertexDataList {
//     /// Length of the list
//     const LENGTH: usize;

//     /// Build buffers from data.
//     fn build<B>(
//         self,
//         factory: &mut Factory<B>,
//         output: &mut Vec<VertexBuffer<B>>,
//     ) -> Result<(), Error>
//     where
//         B: Backend;
// }

// /// Empty list implementation
// impl VertexDataList for () {
//     const LENGTH: usize = 0;
//     fn build<B>(
//         self,
//         _factory: &mut Factory<B>,
//         _output: &mut Vec<VertexBuffer<B>>,
//     ) -> Result<(), Error>
//     where
//         B: Backend,
//     {
//         Ok(())
//     }
// }

// /// Non-empty list implementation
// impl<D, V, L> VertexDataList for (Data<D, V>, L)
// where
//     D: AsRef<[V]>,
//     V: AsVertexFormat,
//     L: VertexDataList,
// {
//     const LENGTH: usize = 1 + L::LENGTH;
//     fn build<B>(
//         self,
//         factory: &mut Factory<B>,
//         output: &mut Vec<VertexBuffer<B>>,
//     ) -> Result<(), Error>
//     where
//         B: Backend,
//     {
//         let (head, tail) = self;
//         output.push(head.build_vertex(factory)?);
//         tail.build(factory, output)
//     }
// }

// /// Optional index data type.
// pub trait IndexDataMaybe {
//     /// Build buffer (or don't) from data.
//     fn build<B>(
//         self,
//         factory: &mut Factory<B>,
//     ) -> Result<Option<IndexBuffer<B>>, Error>
//     where
//         B: Backend;
// }

// /// None index data implementation
// impl IndexDataMaybe for () {
//     /// No data - no buffer
//     fn build<B>(
//         self,
//         _factory: &mut Factory<B>,
//     ) -> Result<Option<IndexBuffer<B>>, Error>
//     where
//         B: Backend,
//     {
//         Ok(None)
//     }
// }

// impl<D> IndexDataMaybe for Data<D, u16>
// where
//     D: AsRef<[u16]>,
// {
//     /// Build `u16` index buffer.
//     fn build<B>(
//         self,
//         factory: &mut Factory<B>,
//     ) -> Result<Option<IndexBuffer<B>>, Error>
//     where
//         B: Backend,
//     {
//         self.build_index(factory).map(Some)
//     }
// }

// impl<D> IndexDataMaybe for Data<D, u32>
// where
//     D: AsRef<[u32]>,
// {
//     /// Build `u32` index buffer.
//     fn build<B>(
//         self,
//         factory: &mut Factory<B>,
//     ) -> Result<Option<IndexBuffer<B>>, Error>
//     where
//         B: Backend,
//     {
//         self.build_index(factory)
//             .map(Some)
//     }
// }


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

/// Abstracts over two types of indices and their absence.
pub enum Indices<'a> {
    None,
    U16(Cow<'a, [u16]>),
    U32(Cow<'a, [u32]>),
}

impl From<Vec<u16>> for Indices<'static> {
    fn from(vec: Vec<u16>) -> Self {
        Indices::U16(vec.into())
    }
}

impl<'a> From<&'a [u16]> for Indices<'a> {
    fn from(slice: &'a [u16]) -> Self {
        Indices::U16(slice.into())
    }
}

impl<'a> From<Cow<'a, [u16]>> for Indices<'a> {
    fn from(cow: Cow<'a, [u16]>) -> Self {
        Indices::U16(cow)
    }
}

impl From<Vec<u32>> for Indices<'static> {
    fn from(vec: Vec<u32>) -> Self {
        Indices::U32(vec.into())
    }
}

impl<'a> From<&'a [u32]> for Indices<'a> {
    fn from(slice: &'a [u32]) -> Self {
        Indices::U32(slice.into())
    }
}

impl<'a> From<Cow<'a, [u32]>> for Indices<'a> {
    fn from(cow: Cow<'a, [u32]>) -> Self {
        Indices::U32(cow)
    }
}

/// Generics-free mesh builder.
/// Useful for creating mesh from non-predefined set of data.
/// Like from glTF.
#[derive(Clone, Debug)]
pub struct MeshBuilder<'a> {
    vertices: SmallVec<[(Cow<'a, [u8]>, VertexFormat<'static>); 16]>,
    indices: Option<(Cow<'a, [u8]>, IndexType)>,
    prim: Primitive,
    transform: Matrix4<f32>,
}

impl<'a> MeshBuilder<'a> {
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
        I: Into<Indices<'a>>,
    {
        self.indices = match indices.into() {
            Indices::None => None,
            Indices::U16(i) => Some((cast_cow(i), IndexType::U16)),
            Indices::U32(i) => Some((cast_cow(i), IndexType::U32)),
        };
        self
    }

    /// Add another vertices to the `MeshBuilder`
    pub fn with_vertices<V, D>(mut self, vertices: D) -> Self
    where
        V: AsVertexFormat + 'a,
        D: Into<Cow<'a, [V]>>
    {
        self.vertices.push((cast_cow(vertices.into()), V::VERTEX_FORMAT));
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
        &self,
        factory: &mut Factory<B>,
    ) -> Result<Mesh<B>, Error>
    where
        B: Backend,
    {
        Ok(Mesh {
            vbufs: self.vertices
                .iter()
                .map(|&(ref vertices, format)| {
                    let len = vertices.len() as VertexCount / format.stride as VertexCount;
                    Ok(VertexBuffer {
                        buffer: {
                            let mut buffer = factory.create_buffer(
                                Properties::DEVICE_LOCAL,
                                vertices.len() as _,
                                Usage::VERTEX | Usage::TRANSFER_DST,
                            )?;
                            factory.upload_buffer(&mut buffer, 0, &vertices)?;
                            buffer
                        },
                        format,
                        len,
                    })
                })
                .collect::<Result<_, Error>>()?,
            ibuf: match self.indices {
                None => None,
                Some((ref indices, index_type)) => {
                    let stride = match index_type {
                        IndexType::U16 => size_of::<u16>(),
                        IndexType::U32 => size_of::<u32>(),
                    };
                    let len = indices.len() as IndexCount / stride as IndexCount;
                    Some(IndexBuffer {
                        buffer: {
                            let mut buffer = factory.create_buffer(
                                Properties::DEVICE_LOCAL,
                                indices.len() as _,
                                Usage::INDEX | Usage::TRANSFER_DST,
                            )?;
                            factory.upload_buffer(&mut buffer, 0, &indices)?;
                            buffer
                        },
                        index_type,
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
    pub fn new() -> MeshBuilder<'static> {
        MeshBuilder::new()
    }

    /// Primitive type of the `Mesh`
    pub fn primitive(&self) -> Primitive {
        self.prim
    }

    /// Bind buffers to specified attribute locations.
    pub fn bind<'a>(
        &'a self,
        formats: &'a [VertexFormat<'a>],
        vertex: &mut VertexBufferSet<'a, B>,
    ) -> Result<Bind<'a, B>, IncompatibleError> {
        debug_assert!(is_slice_sorted(formats));
        debug_assert!(is_slice_sorted_by_key(&self.vbufs, |vbuf| vbuf.format));
        debug_assert!(vertex.0.is_empty());

        let mut next = 0;
        let mut vertex_count = None;
        for format in formats {
            if let Some(index) = find_compatible_buffer(&self.vbufs[next..], format) {
                // Ensure buffer is valid
                vertex.0.push((self.vbufs[index].buffer.raw(), 0));
                next = index + 1;
                assert!(vertex_count.is_none() || vertex_count == Some(self.vbufs[index].len));
                vertex_count = Some(self.vbufs[index].len);
            } else {
                // Can't bind
                return Err(IncompatibleError);
            }
        }
        Ok(self.ibuf
            .as_ref()
            .map(|ibuf| {
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
            }))
    }

    pub fn dispose(self, factory: &mut Factory<B>) {
        if let Some(ibuf) = self.ibuf {
            factory.destroy_buffer(ibuf.buffer);
        }

        for vbuf in self.vbufs {
            factory.destroy_buffer(vbuf.buffer);
        }
    }

    pub fn transform(&self) -> &Matrix4<f32> {
        &self.transform
    }
}

pub struct IncompatibleError;

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
    pub fn draw(self, vertex: VertexBufferSet<B>, encoder: &mut RenderSubpassCommon<B>) {
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
    debug_assert!(is_slice_sorted_by_key(format.attributes, |a| a.offset));
    for (i, vbuf) in vbufs.iter().enumerate() {
        debug_assert!(is_slice_sorted_by_key(&vbuf.format.attributes, |a| a.offset));
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

    // Don't start searching from index 0 because attributes are sorted
    let mut skip = 0;
    right.attributes.iter().all(|r| {
        left.attributes[skip..]
            .iter()
            .position(|l| *l == *r)
            .map_or(false, |p| {
                skip += p;
                true
            })
    })
}
