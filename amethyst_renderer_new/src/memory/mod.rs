

use std::collections::VecDeque;
use std::cmp::min;
use std::ops::{Deref, DerefMut};

use gfx_hal::{Backend, Device, MemoryType};
use gfx_hal::memory::{Pod, Properties, Requirements};
use gfx_hal::buffer::{Usage as BufferUsage, ViewError, complete_requirements};
use gfx_hal::format::Format;
use gfx_hal::image::{Kind, Level, Usage as ImageUsage};
use gfx_hal::device::{BindError, OutOfMemory};
use gfx_hal::mapping::Error as MappingError;

use allocator::{AllocationType, Allocator, Block, SmartAllocator, shift_for_alignment};
use epoch::{CurrentEpoch, Epoch};

error_chain! {
    foreign_links {
        BindError(BindError);
        ViewError(ViewError);
        BufferCreationError(::gfx_hal::buffer::CreationError);
        ImageCreationError(::gfx_hal::image::CreationError);
        OutOfMemory(::gfx_hal::device::OutOfMemory);
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
    }
}

impl From<MappingError> for Error {
    fn from(error: MappingError) -> Error {
        match error {
            MappingError::InvalidAccess => ErrorKind::InvalidAccess.into(),
            MappingError::OutOfBounds => ErrorKind::OutOfBounds.into(),
            MappingError::OutOfMemory => OutOfMemory.into(),
        }
    }
}


pub struct Epochal<B: Backend, T> {
    inner: T,
    block: Block<B, <SmartAllocator<B> as Allocator<B>>::Tag>,
    properties: Properties,
    requirements: Requirements,
    valid_through: Epoch,
}

impl<B, T> Deref for Epochal<B, T>
where
    B: Backend,
{
    type Target = T;
    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<B, T> DerefMut for Epochal<B, T>
where
    B: Backend,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

struct EpochalDeletionQueue<B: Backend, T> {
    offset: u64,
    queue: VecDeque<Vec<Epochal<B, T>>>,
    clean_vecs: Vec<Vec<Epochal<B, T>>>,
}

impl<B, T> EpochalDeletionQueue<B, T>
where
    B: Backend,
{
    fn add(&mut self, item: Epochal<B, T>) {
        let index = (item.valid_through.0 - self.offset) as usize;
        let ref mut queue = self.queue;
        let ref mut clean_vecs = self.clean_vecs;

        let len = queue.len();
        queue.extend((len..index).map(|_| {
            clean_vecs.pop().unwrap_or_else(|| Vec::new())
        }));
        queue[index].push(item);
    }

    fn clean<F>(&mut self, current: &CurrentEpoch, mut f: F)
    where
        F: FnMut(Epochal<B, T>),
    {
        let index = (current.now().0 - self.offset) as usize;
        let len = self.queue.len();

        for mut vec in self.queue.drain(..min(index, len)) {
            for item in vec.drain(..) {
                f(item);
            }
            self.clean_vecs.push(vec);
        }
        self.offset += index as u64;
    }
}


pub type Buffer<B: Backend> = Epochal<B, B::Buffer>;
pub type Image<B: Backend> = Epochal<B, B::Image>;


pub struct BufferManager<B: Backend> {
    allocator: SmartAllocator<B>,

    buffer_deletion_queue: EpochalDeletionQueue<B, B::Buffer>,
    image_deletion_queue: EpochalDeletionQueue<B, B::Image>,
}

impl<B> BufferManager<B>
where
    B: Backend,
{
    fn create_buffer(
        &mut self,
        device: &B::Device,
        size: u64,
        stride: u64,
        usage: BufferUsage,
        properties: Properties,
        transient: bool,
    ) -> Result<Buffer<B>> {
        let ubuf = device.create_buffer(size, stride, usage)?;
        let requirements = complete_requirements::<B>(device, &ubuf, usage);
        let ty = if transient {
            AllocationType::Arena
        } else {
            AllocationType::Chunk
        };
        let block = self.allocator.alloc(device, (ty, properties), requirements)?;
        let buf = device
            .bind_buffer_memory(
                block.memory(),
                shift_for_alignment(requirements.alignment, block.range().start),
                ubuf,
            )
            .unwrap();
        Ok(Epochal {
            inner: buf,
            block,
            properties,
            requirements,
            valid_through: Epoch::new(),
        })
    }

    fn create_image(
        &mut self,
        device: &B::Device,
        kind: Kind,
        level: Level,
        format: Format,
        usage: ImageUsage,
        properties: Properties,
    ) -> Result<Image<B>> {
        let uimg = device.create_image(kind, level, format, usage)?;
        let requirements = device.get_image_requirements(&uimg);
        let block = self.allocator.alloc(
            device,
            (AllocationType::Chunk, properties),
            requirements,
        )?;
        let img = device
            .bind_image_memory(
                block.memory(),
                shift_for_alignment(requirements.alignment, block.range().start),
                uimg,
            )
            .unwrap();
        Ok(Epochal {
            inner: img,
            block,
            properties,
            requirements,
            valid_through: Epoch::new(),
        })
    }

    fn destroy_buffer(&mut self, buffer: Buffer<B>) {
        self.buffer_deletion_queue.add(buffer);
    }

    fn destroy_image(&mut self, image: Image<B>) {
        self.image_deletion_queue.add(image);
    }

    fn cleanup(&mut self, device: &B::Device, current: &CurrentEpoch) {
        let ref mut allocator = self.allocator;
        self.image_deletion_queue.clean(current, |image| {
            assert!(current.now() > image.valid_through);
            device.destroy_image(image.inner);
            allocator.free(device, image.block);
        });
        self.buffer_deletion_queue.clean(current, |buffer| {
            assert!(current.now() > buffer.valid_through);
            device.destroy_buffer(buffer.inner);
            allocator.free(device, buffer.block);
        });
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
