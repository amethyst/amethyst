//! Misc. rendy and rendering utility functions and types.
use core::{
    hash::Hash,
    iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator},
    ops::{Add, Range},
};

use amethyst_core::num::PrimInt;
use derivative::Derivative;
use glsl_layout::*;
use rendy::{
    factory::Factory,
    graph::render::PrepareResult,
    hal::{self, buffer::Usage, format, pso},
    memory::MemoryUsage,
    mesh::VertexFormat,
    resource::{BufferCreationError, BufferInfo, Escape, SubRange},
};
use smallvec::SmallVec;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::types::{Backend, Texture};

/// Helper function to clone ranges.
#[inline]
pub fn next_range<T: Add<Output = T> + Clone>(prev: &Range<T>, length: T) -> Range<T> {
    prev.end.clone()..prev.end.clone() + length
}

/// Helper function to convert `Range` to a `SubRange`
#[inline]
pub fn sub_range(range: Range<u64>) -> SubRange {
    SubRange {
        offset: range.start,
        size: Some(range.end - range.start),
    }
}

/// Helper function to convert `Range` types.
#[inline]
pub fn usize_range(range: Range<u64>) -> Range<usize> {
    range.start as usize..range.end as usize
}

/// This function is used extensively to ensure buffers are allocated and sized appropriately to
/// their use. This function will either allocate a new buffer, resize the current buffer, or perform
/// no action depending on the needs of the function call. This can be used for dynamic buffer
/// allocation or single static buffer allocation.
pub fn ensure_buffer<B: Backend>(
    factory: &Factory<B>,
    buffer: &mut Option<Escape<rendy::resource::Buffer<B>>>,
    usage: Usage,
    memory_usage: impl MemoryUsage,
    min_size: u64,
) -> Result<bool, BufferCreationError> {
    #[cfg(feature = "profiler")]
    profile_scope!("ensure_buffer");

    if buffer.as_ref().map(|b| b.size()).unwrap_or(0) < min_size {
        let new_size = min_size.next_power_of_two();
        let new_buffer = factory.create_buffer(
            BufferInfo {
                size: new_size,
                usage,
            },
            memory_usage,
        )?;
        *buffer = Some(new_buffer);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Helper function for memory alignment.
pub fn align_size<T: Uniform>(align: u64, array_len: usize) -> u64
where
    T::Std140: Sized,
{
    let size = (core::mem::size_of::<T::Std140>() * array_len) as u64;
    ((size + align - 1) / align) * align
}

/// Helper function to create a `GraphicsShaderSet`
pub fn simple_shader_set<'a, B: Backend>(
    vertex: &'a B::ShaderModule,
    fragment: Option<&'a B::ShaderModule>,
) -> pso::GraphicsShaderSet<'a, B> {
    simple_shader_set_ext(vertex, fragment, None, None, None)
}

/// Helper function to create a `GraphicsShaderSet`
pub fn simple_shader_set_ext<'a, B: Backend>(
    vertex: &'a B::ShaderModule,
    fragment: Option<&'a B::ShaderModule>,
    hull: Option<&'a B::ShaderModule>,
    domain: Option<&'a B::ShaderModule>,
    geometry: Option<&'a B::ShaderModule>,
) -> pso::GraphicsShaderSet<'a, B> {
    fn map_entry_point<B: Backend>(module: &B::ShaderModule) -> pso::EntryPoint<'_, B> {
        pso::EntryPoint {
            entry: "main",
            module,
            specialization: pso::Specialization::default(),
        }
    }

    pso::GraphicsShaderSet {
        vertex: map_entry_point(vertex),
        fragment: fragment.map(map_entry_point),
        hull: hull.map(map_entry_point),
        domain: domain.map(map_entry_point),
        geometry: geometry.map(map_entry_point),
    }
}

/// Helper function which takes an array of vertex format information and returns allocated
/// `VertexBufferDesc` and `AttributeDesc` collections.
pub fn vertex_desc(
    formats: &[(VertexFormat, pso::VertexInputRate)],
) -> (Vec<pso::VertexBufferDesc>, Vec<pso::AttributeDesc>) {
    let mut vertex_buffers = Vec::with_capacity(formats.len());
    let mut attributes = Vec::with_capacity(formats.len());

    let mut sorted: SmallVec<[_; 16]> = formats.iter().enumerate().collect();
    sorted.sort_unstable_by(|a, b| a.1.cmp(&b.1));

    let mut loc_offset = 0;
    for (loc_base, (format, rate)) in sorted {
        push_vertex_desc(
            format.gfx_vertex_input_desc(*rate),
            loc_base as pso::Location + loc_offset,
            &mut vertex_buffers,
            &mut attributes,
        );
        loc_offset += format.attributes.len() as pso::Location - 1;
    }
    (vertex_buffers, attributes)
}

/// Helper function which takes an iterator of tuple-stored vertex buffer descriptions and writes
/// into `VertexBufferDesc` and `AttributeDesc` collections.
pub fn push_vertex_desc(
    (elements, stride, rate): (
        impl IntoIterator<Item = pso::Element<format::Format>>,
        pso::ElemStride,
        pso::VertexInputRate,
    ),
    mut location: pso::Location,
    vertex_buffers: &mut Vec<pso::VertexBufferDesc>,
    attributes: &mut Vec<pso::AttributeDesc>,
) {
    let index = vertex_buffers.len() as pso::BufferIndex;
    vertex_buffers.push(pso::VertexBufferDesc {
        binding: index,
        stride,
        rate,
    });

    for element in elements.into_iter() {
        attributes.push(pso::AttributeDesc {
            location,
            binding: index,
            element,
        });
        location += 1;
    }
}

/// Helper function to create a `DescriptorSetWrite` from arguments
#[inline]
pub fn desc_write<'a, B: Backend>(
    set: &'a B::DescriptorSet,
    binding: u32,
    descriptor: pso::Descriptor<'a, B>,
) -> pso::DescriptorSetWrite<'a, B, Option<pso::Descriptor<'a, B>>> {
    pso::DescriptorSetWrite {
        set,
        binding,
        array_offset: 0,
        descriptors: Some(descriptor),
    }
}

/// Helper function to create a `CombinedImageSampler` from a supplied `Texture` and `Layout`
#[inline]
pub fn texture_desc<B: Backend>(
    texture: &Texture,
    layout: hal::image::Layout,
) -> Option<pso::Descriptor<'_, B>> {
    B::unwrap_texture(texture).map(|inner| {
        pso::Descriptor::CombinedImageSampler(inner.view().raw(), layout, inner.sampler().raw())
    })
}

/// Combines an iterator of descriptor information in tuple form into a `DescriptorSetLayoutBinding`
/// # Limitations
/// * All descriptors are created as single count and immutable_samplers is false.
pub fn set_layout_bindings(
    bindings: impl IntoIterator<Item = (u32, pso::DescriptorType, pso::ShaderStageFlags)>,
) -> Vec<pso::DescriptorSetLayoutBinding> {
    bindings
        .into_iter()
        .flat_map(|(times, ty, stage_flags)| (0..times).map(move |_| (ty, stage_flags)))
        .enumerate()
        .map(|(binding, (ty, stage_flags))| {
            pso::DescriptorSetLayoutBinding {
                binding: binding as u32,
                ty,
                count: 1,
                stage_flags,
                immutable_samplers: false,
            }
        })
        .collect()
}

/// Helper forward lookup struct using `FnvHashMap`
#[derive(Debug, Default)]
pub struct LookupBuilder<I: Hash + Eq> {
    forward: fnv::FnvHashMap<I, usize>,
    len: usize,
}

impl<I: Hash + Eq> LookupBuilder<I> {
    /// Create a new `LookupBuilder`
    pub fn new() -> LookupBuilder<I> {
        LookupBuilder {
            forward: fnv::FnvHashMap::default(),
            len: 0,
        }
    }

    /// Return or insert the supplied Id from the table.
    pub fn forward(&mut self, id: I) -> usize {
        if let Some(&id_num) = self.forward.get(&id) {
            id_num
        } else {
            let id_num = self.len;
            self.forward.insert(id, id_num);
            self.len += 1;
            id_num
        }
    }
}

/// Convert any type slice to bytes slice.
pub fn slice_as_bytes<T>(slice: &[T]) -> &[u8] {
    unsafe {
        // Inspecting any value as bytes should be safe.
        core::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            core::mem::size_of::<T>() * slice.len(),
        )
    }
}

/// Copy the byte-data from an iterator into a slice
pub fn write_into_slice<I: IntoIterator>(dst_slice: &mut [u8], iter: I) {
    for (data, dst) in iter
        .into_iter()
        .zip(dst_slice.chunks_exact_mut(std::mem::size_of::<I::Item>()))
    {
        dst.copy_from_slice(slice_as_bytes(&[data]));
    }
}

/// Iterator counting adapter.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[allow(missing_debug_implementations)]
pub struct TapCountIterator<'a, T: PrimInt, I: Iterator> {
    inner: I,
    counter: &'a mut T,
}

/// Iterator counting adapter.
pub trait TapCountIter {
    /// The inner iterator type for this access counter.
    type Iter: Iterator;
    /// Implemented for counting iterator access.
    fn tap_count<T: PrimInt>(self, counter: &mut T) -> TapCountIterator<'_, T, Self::Iter>;
}

impl<I: Iterator> TapCountIter for I {
    type Iter = I;
    fn tap_count<T: PrimInt>(self, counter: &mut T) -> TapCountIterator<'_, T, I> {
        TapCountIterator {
            inner: self,
            counter,
        }
    }
}

impl<'a, T: PrimInt, I: Iterator> Iterator for TapCountIterator<'a, T, I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|d| {
            *self.counter = *self.counter + T::one();
            d
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T: PrimInt, I: DoubleEndedIterator> DoubleEndedIterator for TapCountIterator<'a, T, I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|d| {
            *self.counter = *self.counter + T::one();
            d
        })
    }
}

impl<'a, T: PrimInt, I: ExactSizeIterator> ExactSizeIterator for TapCountIterator<'a, T, I> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a, T: PrimInt, I: FusedIterator> FusedIterator for TapCountIterator<'a, T, I> {}

/// Helper structure for tracking indexed changes for per-image draw call recording.
#[derive(Debug, Clone, Copy, Derivative)]
#[derivative(Default)]
pub enum ChangeDetection {
    /// Has not changed, considered stable
    #[derivative(Default)]
    Stable,
    /// Change occurred, index of value
    Changed(usize),
}

impl ChangeDetection {
    /// Returns true if recording is not needed and the image can be re-used.
    pub fn can_reuse(&mut self, index: usize, changed: bool) -> bool {
        use ChangeDetection::*;
        match (*self, changed) {
            (_, true) => {
                *self = Changed(index);
                false
            }
            (Changed(idx), false) if idx == index => {
                *self = Stable;
                true
            }
            (Stable, false) => true,
            (Changed(_), false) => false,
        }
    }

    /// Returns the proper `PrepareResult` case using `can_reuse`
    pub fn prepare_result(&mut self, index: usize, changed: bool) -> PrepareResult {
        if self.can_reuse(index, changed) {
            PrepareResult::DrawReuse
        } else {
            PrepareResult::DrawRecord
        }
    }
}
