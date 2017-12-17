//!
//! Memory management for rendering.
//! 

use std::cmp::Eq;
use std::fmt::Debug;
use std::ops::{Add, Rem, Sub};

use gfx_hal::Backend;
use gfx_hal::buffer::Usage as BufferUsage;
use gfx_hal::image::Usage as ImageUsage;
use gfx_hal::memory::{Pod, Properties, Requirements};

mod arena;
mod block;
mod chunked;
mod combined;
mod allocator;
mod root;
mod smart;

pub use self::allocator::{Allocator, Buffer, Image, WeakBuffer, WeakImage};
use self::block::Block;


error_chain! {
    foreign_links {
        BindError(::gfx_hal::device::BindError);
        // ViewError(::gfx_hal::buffer::ViewError);
        BufferCreationError(::gfx_hal::buffer::CreationError);
        ImageCreationError(::gfx_hal::image::CreationError);
        OutOfMemory(::gfx_hal::device::OutOfMemory);
    }

    errors {
        NoCompatibleMemoryType {}
        BufferUsageAndProperties(usage: BufferUsage, properties: Properties) {
            description("No memory type supports both usage and properties specified")
            display("No memory type supports both
                usage ({:?}) and properties ({:?}) specified", usage, properties)
        }
        ImageUsageAndProperties(usage: ImageUsage, properties: Properties) {
            description("No memory type supports both usage and properties specified")
            display("No memory type supports both
                usage ({:?}) and properties ({:?}) specified", usage, properties)
        }
    }
}


pub trait MemoryAllocator<B: Backend> {
    type Info;
    type Tag: Debug + Copy + Send + Sync;

    fn alloc(
        &mut self,
        device: &B::Device,
        info: Self::Info,
        reqs: Requirements,
    ) -> Result<Block<B, Self::Tag>>;
    fn free(&mut self, device: &B::Device, block: Block<B, Self::Tag>);
    fn is_unused(&self) -> bool;
    fn dispose(self, device: &B::Device);
}

pub trait MemorySubAllocator<B: Backend> {
    type Owner;
    type Info;
    type Tag: Debug + Copy + Send + Sync;

    fn alloc(
        &mut self,
        owner: &mut Self::Owner,
        device: &B::Device,
        info: Self::Info,
        reqs: Requirements,
    ) -> Result<Block<B, Self::Tag>>;
    fn free(&mut self, owner: &mut Self::Owner, device: &B::Device, block: Block<B, Self::Tag>);
    fn is_unused(&self) -> bool;
    fn dispose(self, owner: &mut Self::Owner, device: &B::Device);
}


pub fn calc_alignment_shift<T>(alignment: T, offset: T) -> T
where
    T: From<u8> + Add<Output = T> + Sub<Output = T> + Rem<Output = T> + Eq + Copy,
{
    if offset == 0.into() {
        0.into()
    } else {
        alignment - (offset - 1.into()) % alignment - 1.into()
    }
}


pub fn shift_for_alignment<T>(alignment: T, offset: T) -> T
where
    T: From<u8> + Add<Output = T> + Sub<Output = T> + Rem<Output = T> + Eq + Copy,
{
    offset + calc_alignment_shift(alignment, offset)
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
