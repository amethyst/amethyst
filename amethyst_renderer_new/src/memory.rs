use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use gfx_hal::{Backend, Device, MemoryType};
use gfx_hal::buffer::{Usage as BufferUsage, ViewError};
use gfx_hal::device::{BindError, OutOfMemory};
use gfx_hal::format::Format;
use gfx_hal::image::{Kind, Level, Usage as ImageUsage};
use gfx_hal::mapping::Error as MappingError;
use gfx_hal::memory::{Pod, Properties, Requirements};

use std::result::Result as StdResult;

error_chain! {
    foreign_links {
        BindError(BindError);
        ViewError(ViewError);
        BufferCreationError(::gfx_hal::buffer::CreationError);
        ImageCreationError(::gfx_hal::image::CreationError);
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
        device: &B::Device,
        size: usize,
        stride: usize,
        usage: BufferUsage,
        fill: Option<&[u8]>,
    ) -> Result<B::Buffer>;

    fn allocate_image(
        &mut self,
        device: &B::Device,
        kind: Kind,
        level: Level,
        format: Format,
        usage: ImageUsage,
    ) -> Result<B::Image>;

    fn commit_transfers(&mut self, cbuf: &mut B::CommandBuffer) {}
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
    fn allocate(&self, size: usize, alignment: usize) -> Option<(&B::Memory, usize)> {
        let pos = self.allocated
            .fetch_add(size + alignment, Ordering::Acquire) + alignment;
        if self.size - size >= pos {
            let shift = pos % alignment;
            let pos = pos - shift;
            Some((&self.memory, pos))
        } else {
            None
        }
    }
}

struct DumbAllocatorNodes<B: Backend> {
    memory_type: MemoryType,
    nodes: Vec<Arc<DumbAllocatorNode<B>>>,
}

/// This allocator is do dumb it can't even free memory
pub struct DumbAllocator<B: Backend> {
    granularity: usize,
    nodes: Vec<DumbAllocatorNodes<B>>,
    commits: Vec<Box<Fn(&mut B::CommandBuffer)>>,
}

impl<B> DumbAllocator<B>
where
    B: Backend,
{
    pub fn new(mut memory_types: Vec<MemoryType>) -> Self {
        DumbAllocator {
            granularity: 268_435_456, // 256 MB
            nodes: memory_types.into_iter().map(|t| DumbAllocatorNodes { memory_type: t, nodes: vec![] }).collect(),
            commits: vec![],
        }
    }

    fn find_nodes(&mut self, type_mask: u64, properties: Properties) -> &mut DumbAllocatorNodes<B> {
        self.nodes.iter_mut().find(|nodes| {
            type_mask & (1 << nodes.memory_type.id) != 0 && nodes.memory_type.properties.contains(properties)
        }).unwrap()
    }

    fn allocate_buffer_unfilled(
        &mut self,
        device: &B::Device,
        size: usize,
        stride: usize,
        usage: BufferUsage,
        properties: Properties,
    ) -> Result<B::Buffer> {
        let ubuf = device.create_buffer(size as u64, stride as u64, usage)?;

        let Requirements {
            size,
            alignment,
            type_mask,
        } = device.get_buffer_requirements(&ubuf);

        println!("Creating buffer. type_mask<{:b}>", type_mask);

        if size > usize::max_value() as u64 || alignment > usize::max_value() as u64 {
            return Err(ErrorKind::OutOfMemory.into());
        }

        let size = size as usize;
        let alignment = alignment as usize;

        let granularity = self.granularity;
        let nodes = self.find_nodes(type_mask, properties);
        let memory_type = nodes.memory_type;
        let ref mut nodes = nodes.nodes;

        let ubuf = match nodes
            .last()
            .and_then(|node| node.allocate(size, alignment))
        {
            Some((memory, offset)) => {
                return Ok(device.bind_buffer_memory(memory, offset as u64, ubuf)?)
            }
            None => ubuf,
        };

        let node_size = (size + alignment - 1) / granularity + granularity;

        let node = DumbAllocatorNode {
            memory: device.allocate_memory(
                &memory_type,
                node_size as u64,
            )?,
            size: node_size,
            allocated: AtomicUsize::new(0),
            freed: AtomicUsize::new(0),
        };
        nodes.push(Arc::new(node));

        let (memory, offset) = nodes
            .last()
            .unwrap()
            .allocate(size, alignment)
            .expect("Hey!");
        Ok(device.bind_buffer_memory(memory, offset as u64, ubuf)?)
    }

    fn allocate_image_unfilled(
        &mut self,
        device: &B::Device,
        kind: Kind,
        level: Level,
        format: Format,
        usage: ImageUsage,
        properties: Properties,
    ) -> Result<B::Image> {
        let uimg = device.create_image(kind, level, format, usage)?;

        let Requirements {
            size,
            alignment,
            type_mask,
        } = device.get_image_requirements(&uimg);

        println!("Creating image. type_mask<{:b}>", type_mask);

        if size > usize::max_value() as u64 || alignment > usize::max_value() as u64 {
            return Err(ErrorKind::OutOfMemory.into());
        }

        let size = size as usize;
        let alignment = alignment as usize;

        let granularity = self.granularity;
        let nodes = self.find_nodes(type_mask, properties);
        let memory_type = nodes.memory_type;
        let ref mut nodes = nodes.nodes;

        let uimg = match nodes
            .last()
            .and_then(|node| node.allocate(size, alignment))
        {
            Some((memory, offset)) => {
                return Ok(device.bind_image_memory(memory, offset as u64, uimg)?)
            }
            None => uimg,
        };

        let node_size = (size + alignment - 1) / granularity + granularity;

        let node = DumbAllocatorNode {
            memory: device.allocate_memory(
                &memory_type,
                node_size as u64,
            )?,
            size: node_size,
            allocated: AtomicUsize::new(0),
            freed: AtomicUsize::new(0),
        };
        nodes.push(Arc::new(node));

        let (memory, offset) = nodes
            .last()
            .unwrap()
            .allocate(size, alignment)
            .expect("Hey!");
        Ok(device.bind_image_memory(memory, offset as u64, uimg)?)
    }
}

impl<B> Allocator<B> for DumbAllocator<B>
where
    B: Backend,
{
    fn allocate_buffer(
        &mut self,
        device: &B::Device,
        size: usize,
        stride: usize,
        usage: BufferUsage,
        fill: Option<&[u8]>,
    ) -> Result<B::Buffer> {
        let buffer = self.allocate_buffer_unfilled(device, size, stride, usage, Properties::CPU_VISIBLE | Properties::COHERENT)?;
        match fill {
            Some(data) => {
                let mut writer = device
                    .acquire_mapping_writer::<u8>(&buffer, 0..data.len() as u64)
                    .map_err(Error::from)?;
                writer.copy_from_slice(data);
                device.release_mapping_writer(writer);
            }
            None => {}
        };
        Ok(buffer)
    }

    fn allocate_image(
        &mut self,
        device: &B::Device,
        kind: Kind,
        level: Level,
        format: Format,
        usage: ImageUsage,
    ) -> Result<B::Image> {
        self.allocate_image_unfilled(device, kind, level, format, usage, Properties::DEVICE_LOCAL)
    }

    fn commit_transfers(&mut self, cbuf: &mut B::CommandBuffer) {
        for commit in self.commits.drain(..) {
            commit(cbuf);
        }
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
