use gfx_hal::{Backend, Device};
use gfx_hal::buffer::Usage as BufferUsage;
use gfx_hal::command::{BufferCopy, CommandBuffer};
use gfx_hal::memory::{Pod, Properties};
use gfx_hal::queue::{Supports, Transfer};

use epoch::{CurrentEpoch, Eh};
use memory::{Allocator, Buffer, Image, WeakBuffer, WeakImage, cast_pod_vec};

const DIRECT_TRESHOLD: u64 = 1024;

error_chain! {
    links {
        Memory(::memory::Error, ::memory::ErrorKind);
    }
    foreign_links {
        MappingError(::gfx_hal::mapping::Error);
    }
}

#[derive(Debug)]
pub enum Upload<B: Backend> {
    BufferStaging {
        dst: WeakBuffer<B>,
        offset: u64,
        src: Buffer<B>,
    },
    BufferDirect {
        dst: WeakBuffer<B>,
        offset: u64,
        data: Vec<u8>,
    },
    ImageStaing,
}

impl<B> Upload<B>
where
    B: Backend,
{
    pub fn new<D, T>(
        allocator: &mut Allocator<B>,
        ec: &CurrentEpoch,
        device: &B::Device,
        dst: &mut Buffer<B>,
        offset: u64,
        data: D,
    ) -> Result<Self>
    where
        D: AsRef<[T]> + Into<Vec<T>>,
        T: Pod,
    {
        let bytes = (data.as_ref().len() * ::std::mem::size_of::<T>()) as u64;
        assert!(dst.get_size() >= bytes + offset);

        if bytes > DIRECT_TRESHOLD {
            let src = allocator.create_buffer(
                device,
                bytes,
                bytes,
                BufferUsage::TRANSFER_SRC,
                Properties::CPU_VISIBLE,
                true,
            )?;

            {
                let mut writer = device.acquire_mapping_writer(src.raw(), 0..bytes)?;
                writer.copy_from_slice(data.as_ref());
                device.release_mapping_writer(writer);
            }

            Ok(Upload::BufferStaging {
                dst: Eh::borrow(dst, ec.now() + 2),
                src,
                offset,
            })
        } else {
            Ok(Upload::BufferDirect {
                dst: Eh::borrow(dst, ec.now() + 2),
                data: cast_pod_vec(data.into()),
                offset,
            })
        }
    }

    pub fn commit<C>(self, cbuf: &mut CommandBuffer<B, C>, ec: &CurrentEpoch)
    where
        C: Supports<Transfer>,
    {
        match self {
            Upload::BufferStaging { dst, offset, src } => {
                cbuf.copy_buffer(
                    src.raw(),
                    dst.get(ec).expect(
                        "Expected to be commited before dst expires",
                    ).raw(),
                    &[
                        BufferCopy {
                            src: 0,
                            dst: offset,
                            size: src.get_size(),
                        },
                    ],
                )
            }
            Upload::BufferDirect { dst, offset, data } => {
                cbuf.update_buffer(
                    dst.get(ec).expect(
                        "Expected to be commited before dst expires",
                    ).raw(),
                    offset,
                    &data,
                )
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub struct Uploader<B: Backend> {
    uploads: Vec<Upload<B>>,
}


impl<B> Uploader<B>
where
    B: Backend,
{
    pub fn new() -> Self {
        Uploader { uploads: Vec::new() }
    }

    pub fn upload_buffer<T, D>(
        &mut self,
        allocator: &mut Allocator<B>,
        ec: &CurrentEpoch,
        device: &B::Device,
        dst: &mut Buffer<B>,
        offset: u64,
        data: D,
    ) -> Result<()>
    where
        D: AsRef<[T]> + Into<Vec<T>>,
        T: Pod,
    {
        Ok(self.uploads.push(Upload::new(
            allocator,
            ec,
            device,
            dst,
            offset,
            data,
        )?))
    }

    pub fn commit<C>(&mut self, cbuf: &mut CommandBuffer<B, C>, ec: &CurrentEpoch)
    where
        C: Supports<Transfer>,
    {
        for upload in self.uploads.drain(..) {
            upload.commit(cbuf, ec);
        }
    }
}
