use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use gfx_hal::{Backend, Device, MemoryType};
use gfx_hal::buffer::{CreationError, Usage, ViewError};
use gfx_hal::device::{BindError, OutOfMemory};
use gfx_hal::mapping::Error as MappingError;
use gfx_hal::memory::{Pod, Properties, Requirements};

use std::result::Result as StdResult;

error_chain! {
    foreign_links {
        BindError(BindError);
        ViewError(ViewError);
    }

    errors {
        InvalidAccess {
            description("Invalid access"),
            display("Invalid access"),
        }
        OutOfBounds {
            description("Out of bounds"),
            display("Out of bounds"),
        }
        OutOfMemory {
            description("Out of memory"),
            display("Out of memory"),
        }
        CreationError(inner: CreationError) {
            description("Buffer creation error"),
            display("Can't create buffer: {:?}", inner),
        }
    }
}

impl From<MappingError> for ErrorKind {
    fn from(error: MappingError) -> ErrorKind {
        match error {
            MappingError::InvalidAccess => ErrorKind::InvalidAccess,
            MappingError::OutOfBounds => ErrorKind::OutOfBounds,
            MappingError::OutOfMemory => ErrorKind::OutOfMemory,
        }
    }
}

impl From<MappingError> for Error {
    fn from(error: MappingError) -> Error {
        ErrorKind::from(error).into()
    }
}

impl From<OutOfMemory> for Error {
    fn from(_: OutOfMemory) -> Error {
        ErrorKind::OutOfMemory.into()
    }
}

/// This trait is used to allocate `Buffer`s from `Memory`
/// It uses `Device` to grab `Memory` in big chunks and then
/// Use it to allocated various size buffers
pub trait Allocator<B: Backend> {
    /// Allocate buffer
    /// TODO: Add options to this function to choose memory types and startegies
    fn allocate_buffer(
        &mut self,
        device: &mut B::Device,
        size: usize,
        stride: usize,
        usage: Usage,
        fill: Option<&[u8]>,
    ) -> Result<B::Buffer>;
}


struct DumbAllocatorNode<B: Backend> {
    memory: B::Memory,
    size: usize,
    allocated: AtomicUsize,
    freed: AtomicUsize,
}

impl<B> DumbAllocatorNode<B>
where
    B: Backend,
{
    fn allocate_buffer(
        &self,
        device: &mut B::Device,
        ubuf: B::UnboundBuffer,
        size: usize,
        alignment: usize,
    ) -> StdResult<B::Buffer, B::UnboundBuffer> {
        let pos = self.allocated.fetch_add(
            size + alignment,
            Ordering::Acquire,
        ) - size;
        if self.size - size >= pos {
            let shift = pos % alignment;
            let pos = pos - shift;
            let buf = device
                .bind_buffer_memory(&self.memory, pos as u64, ubuf)
                .expect("Bounds and types are checked");
            Ok(buf)
        } else {
            Err(ubuf)
        }
    }
}


/// This allocator is do dumb it can't even free memory
pub struct DumbAllocator<B: Backend> {
    granularity: usize,
    nodes: Vec<Arc<DumbAllocatorNode<B>>>,
}

impl<B> DumbAllocator<B>
where
    B: Backend,
{
    pub fn new() -> Self {
        DumbAllocator {
            granularity: 268_435_456, // 256 MB
            nodes: Vec::new(),
        }
    }

    fn allocate_buffer_unfilled(
        &mut self,
        device: &mut B::Device,
        size: usize,
        stride: usize,
        usage: Usage,
    ) -> Result<B::Buffer> {
        let ubuf = device
            .create_buffer(size as u64, stride as u64, usage)
            .map_err(ErrorKind::CreationError)?;

        let Requirements {
            size,
            alignment,
            type_mask,
        } = device.get_buffer_requirements(&ubuf);

        if size > usize::max_value() as u64 || alignment > usize::max_value() as u64 {
            return Err(ErrorKind::OutOfMemory.into());
        }

        let size = size as usize;
        let alignment = alignment as usize;

        let ubuf = if let Some(node) = self.nodes.last() {
            match node.allocate_buffer(device, ubuf, size, alignment) {
                Ok(buf) => return Ok(buf),
                Err(ubuf) => ubuf,
            }
        } else {
            ubuf
        };

        let node_size = (size + alignment - 1) / self.granularity + self.granularity;

        let node = DumbAllocatorNode {
            memory: device.allocate_memory(
                &MemoryType {
                    id: 0,
                    properties: Properties::COHERENT | Properties::CPU_VISIBLE,
                    heap_index: 0,
                },
                node_size as u64,
            )?,
            size: node_size,
            allocated: AtomicUsize::new(0),
            freed: AtomicUsize::new(0),
        };
        self.nodes.push(Arc::new(node));

        Ok(
            self.nodes
                .last()
                .unwrap()
                .allocate_buffer(device, ubuf, size, alignment)
                .expect("Hey!"),
        )
    }
}

impl<B> Allocator<B> for DumbAllocator<B>
where
    B: Backend,
{
    fn allocate_buffer(
        &mut self,
        device: &mut B::Device,
        size: usize,
        stride: usize,
        usage: Usage,
        fill: Option<&[u8]>,
    ) -> Result<B::Buffer> {
        let buffer = self.allocate_buffer_unfilled(device, size, stride, usage)?;
        match fill {
            Some(data) => {
                let mut writer = device
                    .acquire_mapping_writer::<u8>(&buffer, 0..data.len() as u64)
                    .map_err(Error::from)?;
                writer.copy_from_slice(data);
            }
            None => {}
        };
        Ok(buffer)
    }
}


/// Cast `Vec` of one `Pod` type into `Vec` of another `Pod` type
/// Align and size of input type must be divisible by align and size of output type
/// Converting from arbitrary `T: Pod` into `u8` is always possible
/// as `u8` has size and align equal to 1
pub fn cast_pod_vec<T, Y>(mut vec: Vec<T>) -> Vec<Y>
where
    T: Pod,
    Y: Pod,
{
    use std::mem::{align_of, forget, size_of};

    debug_assert_eq!(align_of::<T>() % align_of::<Y>(), 0);
    debug_assert_eq!(size_of::<T>() % size_of::<Y>(), 0);

    let tsize = size_of::<T>();
    let ysize = size_of::<Y>();

    let p = vec.as_mut_ptr();
    let s = vec.len();
    let c = vec.capacity();

    unsafe {
        forget(vec);
        Vec::from_raw_parts(p as *mut Y, (s * tsize) / ysize, (c * tsize) / ysize)
    }
}
