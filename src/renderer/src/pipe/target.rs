//! Render target used for storing 2D pixel representations of 3D scenes.

use {ColorFormat, DepthFormat, Factory, Resources, Result};
use gfx;

/// Target color buffer.
pub type TargetColorBuffer = gfx::handle::RenderTargetView<Resources, ColorFormat>;

/// Target depth buffer.
pub type TargetDepthBuffer = gfx::handle::DepthStencilView<Resources, DepthFormat>;

/// A render target.
///
/// Each render target contains a certain number of color buffers and an
/// optional depth buffer.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Target {
    color_bufs: Vec<TargetColorBuffer>,
    depth_buf: Option<TargetDepthBuffer>,
    size: (u32, u32),
}

impl Target {
    /// Creates a new TargetBuilder with the given name.
    pub fn new<N: Into<String>>(name: N) -> TargetBuilder {
        TargetBuilder::new(name.into())
    }

    /// Returns the width and height of the render target, measured in pixels.
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Returns an array slice of the render target's color buffers.
    pub fn color_bufs(&self) -> &[TargetColorBuffer] {
        self.color_bufs.as_ref()
    }

    /// Returns the render target's depth-stencil buffer, if it has one.
    pub fn depth_buf(&self) -> Option<&TargetDepthBuffer> {
        self.depth_buf.as_ref()
    }
}

impl<D> From<(Vec<TargetColorBuffer>, D, (u32, u32))> for Target
    where D: Into<Option<TargetDepthBuffer>>
{
    fn from(target: (Vec<TargetColorBuffer>, D, (u32, u32))) -> Target {
        Target {
            color_bufs: target.0,
            depth_buf: target.1.into(),
            size: target.2,
        }
    }
}

/// Builds new render targets.
///
/// By default, it creates render targets with one color buffer and no
/// depth-stencil buffer.
pub struct TargetBuilder {
    custom_size: Option<(u32, u32)>,
    name: String,
    has_depth_buf: bool,
    num_color_bufs: u32,
}

impl TargetBuilder {
    /// Creates a new TargetBuilder.
    pub fn new<S: Into<String>>(name: S) -> Self {
        TargetBuilder {
            custom_size: None,
            name: name.into(),
            has_depth_buf: false,
            num_color_bufs: 1,
        }
    }

    /// Sets whether this render target should have a depth-stencil buffer.
    ///
    /// By default, render targets have no depth-stencil buffers included.
    pub fn with_depth_buf(mut self, has_buf: bool) -> Self {
        self.has_depth_buf = has_buf;
        self
    }

    /// Sets how many color buffers the render target will have. This number
    /// must be greater than zero or else `build()` will fail.
    ///
    /// By default, render targets have only one color buffer.
    pub fn with_num_color_bufs(mut self, num: u32) -> Self {
        self.num_color_bufs = num;
        self
    }

    /// Specifies the custom window size.
    pub fn with_size(mut self, size: (u32, u32)) -> Self {
        self.custom_size = Some(size);
        self
    }

    /// Builds and returns the new render target.
    pub fn build(self, win_size: (u32, u32), fac: &mut Factory) -> Result<(String, Target)> {
        use gfx::Factory;

        let size = match self.custom_size {
            Some(s) => s,
            None => win_size,
        };

        let mut color_bufs = Vec::new();
        for _ in 0..self.num_color_bufs {
            let (w, h) = (size.0 as u16, size.1 as u16);
            let (_, _, rt) = fac.create_render_target(w, h)?;
            color_bufs.push(rt);
        }

        let depth_buf = if self.has_depth_buf {
            let (w, h) = (size.0 as u16, size.1 as u16);
            let depth = fac.create_depth_stencil_view_only(w, h)?;
            Some(depth)
        } else {
            None
        };

        let target = Target {
            color_bufs: color_bufs,
            depth_buf: depth_buf,
            size: size,
        };

        Ok((self.name, target))
    }
}

impl<T> From<(T, u32, bool, (u32, u32))> for TargetBuilder
    where T: Into<String>
{
    fn from(builder: (T, u32, bool, (u32, u32))) -> TargetBuilder {
        TargetBuilder::new(builder.0)
            .with_num_color_bufs(builder.1)
            .with_depth_buf(builder.2)
            .with_size(builder.3)
    }
}
