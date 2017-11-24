
use gfx_hal::memory::Pod;
use gfx_hal::buffer::ViewError;
use gfx_hal::device::{BindError, OutOfMemory};
use gfx_hal::mapping::Error as MappingError;

mod allocator;
mod epoch;

use self::allocator::{Block, SmartAllocator, AllocationType};

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


pub struct Buffer<B: Backend> {
    inner: B::Buffer,
    block: Block<B, SmartAllocator<B>::Tag>,
    properties: Properties,
    requirements: Requirements,
}


pub struct Allocator<B: Backend> {
    allocator: SmartAllocator<B>,
    reclamation_queue: VecDeque<Vec<Eh<Buffer<B>>>>,
}


impl<B> Allocator<B>
where
    B: Backend
{
    fn allocate_buffer(&mut self, device: &B::Device, size: u64, stride: u64, usage: BufferUsage, properties: Properties) -> Result<Buffer<B>> {
        let ubuf = device.create_buffer(size, stride, usage)?;
        let requirements = complete_requirements(device, &ubuf, usage);
        let block = self.allocator.alloc(device, (AllocationType::Chunk, properties), requirements)?;
        let buf = device.bind_buffer_memory(unsafe { block.memory }, shift_for_alignment(requirements.alignment, block.offset), ubuf).unwrap();
        Ok(Buffer {
            inner: buf,
            block,
            properties,
            requirements,
        })
    }
}

