//! Graphics encoder type.

use gfx::handle::Manager;
use gfx::memory::Typed;
use gfx::pso::AccessInfo;
use gfx_core::command::ClearColor;
use gfx_core::target::Depth;
use mesh::Mesh;
use mtl::Material;
use pipe::Target;
use pipe::pass::Effect;
use tex::Texture;
use types::{CommandBuffer, Device, Resources};

pub struct Encoder {
    access_info: AccessInfo<Resources>,
    cmd_buf: CommandBuffer,
    handles: Manager<Resources>,
}

impl Encoder {
    /// Submits the commands in this `Encoder`'s internal `CommandBuffer` to the
    /// GPU, so they can be executed.
    pub fn flush(&mut self, dev: &mut Device) {
        use gfx::{CommandBuffer, Device};

        dev.pin_submitted_resources(&self.handles);
        dev.submit(&mut self.cmd_buf, &self.access_info)
            .expect("Submit fail");
        self.cmd_buf.reset();
        self.access_info.clear();
        self.handles.clear();
    }

    /// Clears all color buffers of `target` to the value `val`.
    pub fn clear_color<C>(&mut self, target: &Target, val: C)
        where C: Into<ClearColor> + Copy
    {
        use gfx::CommandBuffer;

        for buf in target.color_bufs() {
            let raw = self.handles.ref_rtv(buf.as_output.raw()).clone();
            self.cmd_buf.clear_color(raw, val.into())
        }
    }

    /// Clears the depth stencil buffer of `target` to the value `val`.
    pub fn clear_depth<D>(&mut self, target: &Target, val: D)
        where D: Into<Depth> + Copy
    {
        use gfx::CommandBuffer;

        if let Some(buf) = target.depth_buf() {
            let raw = self.handles.ref_dsv(buf.as_output.raw()).clone();
            let depth = val.into();
            let stencil = val.into() as u8;
            self.cmd_buf
                .clear_depth_stencil(raw, Some(depth), Some(stencil))
        }
    }

    /// Draws the given `Mesh` with an `Effect`.
    pub fn draw(&mut self, mesh: &Mesh, effect: &Effect) {}
}

impl From<CommandBuffer> for Encoder {
    fn from(cmd_buf: CommandBuffer) -> Encoder {
        Encoder {
            access_info: AccessInfo::new(),
            cmd_buf: cmd_buf,
            handles: Manager::new(),
        }
    }
}
