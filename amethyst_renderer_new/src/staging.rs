
use gfx_hal::command::BufferCopy;
use gfx_hal::RawCommandBuffer;

use back::*;

pub enum StagingBuffer {
    Unstaged {
        size: usize,
        src: Buffer,
        dst: Buffer,
    },
    Staged {
        size: usize,
        src: Buffer,
        dst: Buffer,
    },
    Loaded { size: usize, buffer: Buffer },
}

impl StagingBuffer {
    fn drive(self, cbuf: &mut CommandBuffer) -> Self {
        use self::StagingBuffer::*;
        match self {
            Unstaged { size, src, dst } => {
                cbuf.copy_buffer(
                    &src,
                    &dst,
                    &[
                        BufferCopy {
                            src: 0,
                            dst: 0,
                            size: size as u64,
                        },
                    ],
                );
                Staged { size, src, dst }
            }
            Staged { size, dst, .. } => Loaded { size, buffer: dst },
            loaded => loaded,
        }
    }

    fn buffer(&self) -> &Buffer {
        use self::StagingBuffer::*;
        match *self {
            Unstaged { ref dst, .. } => dst,
            Staged { ref dst, .. } => dst,
            Loaded { ref buffer, .. } => buffer,
        }
    }
}
