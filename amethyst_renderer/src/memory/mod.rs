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


#[derive(Fail, Debug, Clone)]
pub enum Error {
    #[fail(display = "No compatible memory")]
    NoCompatibleMemoryType,

    #[fail(display = "Out of memory")]
    OutOfMemory,

    #[fail(display = "Buffer creation error occured: {}", error)]
    BufferCreation {
        error: ::gfx_hal::buffer::CreationError,
    },

    #[fail(display = "Image creation error occured: {}", error)]
    ImageCreation {
        error: ::gfx_hal::image::CreationError,
    },

    #[fail(display = "No memory type supports both usage ({:?}) and properties ({:?}) specified", usage, properties)]
    BufferUsageAndProperties {
        usage: BufferUsage,
        properties: Properties,
    },

    #[fail(display = "No memory type supports both usage ({:?}) and properties ({:?}) specified", usage, properties)]
    ImageUsageAndProperties {
        usage: ImageUsage,
        properties: Properties,
    },

    #[fail(display = "Unknown backend error")]
    Other,
}

impl From<::gfx_hal::buffer::CreationError> for Error {
    fn from(error: ::gfx_hal::buffer::CreationError) -> Error {
        Error::BufferCreation {
            error,
        }
    }
}

impl From<::gfx_hal::image::CreationError> for Error {
    fn from(error: ::gfx_hal::image::CreationError) -> Error {
        Error::ImageCreation {
            error,
        }
    }
}

impl From<::gfx_hal::device::BindError> for Error {
    fn from(_: ::gfx_hal::device::BindError) -> Error {
        panic!("Something is wrong with allocator. This should never occur.")
    }
}

impl From<::gfx_hal::device::OutOfMemory> for Error {
    fn from(_: ::gfx_hal::device::OutOfMemory) -> Error {
        Error::OutOfMemory
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
    ) -> Result<Block<B, Self::Tag>, Error>;
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
    ) -> Result<Block<B, Self::Tag>, Error>;
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
#[inline(always)]
pub fn cast_vec<A: Pod, B: Pod>(mut vec: Vec<A>) -> Vec<B> {
    use std::mem;

    let raw_len = mem::size_of::<A>().wrapping_mul(vec.len());
    let len = raw_len / mem::size_of::<B>();

    let raw_cap = mem::size_of::<A>().wrapping_mul(vec.capacity());
    let cap = raw_cap / mem::size_of::<B>();

    assert_eq!(raw_len, mem::size_of::<B>().wrapping_mul(len));
    assert_eq!(raw_cap, mem::size_of::<B>().wrapping_mul(cap));
    
    let ptr = vec.as_mut_ptr() as *mut B;
    mem::forget(vec);

    unsafe {
        Vec::from_raw_parts(
            ptr,
            len,
            cap,
        )
    }
}
