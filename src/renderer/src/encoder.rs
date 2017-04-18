//! Graphics encoder type.

use gfx::handle::Manager;
use gfx::pso::{AccessInfo, RawDataSet};
use gfx::texture::CubeFace;
use tex::Texture;
use types::{CommandBuffer, Resources};

/// Type-erased version of `gfx::Encoder`.
pub struct Encoder {
    access_info: AccessInfo<Resources>,
    cmd_buf: CommandBuffer,
    handles: Manager<Resources>,
    raw_data: RawDataSet<Resources>,
}

impl Encoder {
    pub fn update_texture(&mut self, tex: &Texture, face: Option<CubeFace>) {
    }
}

impl From<CommandBuffer> for Encoder {
    fn from(buf: CommandBuffer) -> Encoder {
        Encoder {
            access_info: AccessInfo::new(),
            cmd_buf: buf,
            handles: Manager::new(),
            raw_data: RawDataSet::new(),
        }
    }
}
