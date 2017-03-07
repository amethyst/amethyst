//! Blits a color or depth buffer from one Target onto another.

use {Encoder, Error, Pass, Result, Target};
use pass::Args;

/// Blits a color or depth buffer from one Target onto another.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlitBuffer {
    buf_info: (String, BufferType),
}

impl BlitBuffer {
    /// Blits the color buffer of the given target onto the Stage's target.
    pub fn color_buf<T: Into<String>>(target_name: T, buf_idx: usize) -> Self {
        BlitBuffer {
            target: (target_name.into(), BufferType::Color(buf_idx)),
        }
    }

    /// Blits the depth buffer of the given target onto the Stage's target.
    pub fn depth_buf<T: Into<String>>(target_name: T) -> Self {
        BlitBuffer {
            target: (target_name.into(), BufferType::Depth),
        }
    }
}

impl Pass for BlitBuffer {
    fn init(&mut self, args: &Args) -> Result<()> {
        let Args(fac, targets) = args;

        let (name, buf_type) = self.buf_info;
        let target = targets.get(&name).ok_or(Error::NoSuchTarget(name))?;
        
        let buf = match buf_type {
            BufferType::Color(i) => target.color_bufs().get(i).unwrap(),
            BufferType::Depth => target.depth_buf().unwrap(),
        };

        Ok(())
    }

    fn apply(&self, enc: &mut Encoder, target: &Target, _: f64) {
    }
}

/// Possible types of buffers in a target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BufferType {
    /// A color buffer with the given index.
    Color(usize),
    /// A depth stencil buffer.
    Depth,
}
