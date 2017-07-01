//! Render target used for storing 2D pixel representations of 3D scenes.

use error::Result;
use fnv::FnvHashMap as HashMap;
use std::sync::Arc;
use types::{DepthStencilView, Factory, RenderTargetView, ShaderResourceView};

/// Target color buffer.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorBuffer {
    /// Shader resource view.
    pub as_input: Option<ShaderResourceView<[f32; 4]>>,
    /// Target view.
    pub as_output: RenderTargetView,
}

/// Target depth-stencil buffer.
#[derive(Clone, Debug, PartialEq)]
pub struct DepthBuffer {
    /// Shader resource view.
    pub as_input: Option<ShaderResourceView<f32>>,
    /// Target view.
    pub as_output: DepthStencilView,
}

/// A render target.
///
/// Each render target contains a certain number of color buffers and an
/// optional depth buffer.
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    color_bufs: Vec<ColorBuffer>,
    depth_buf: Option<DepthBuffer>,
    size: (u32, u32),
}

impl Target {
    /// Creates a new TargetBuilder with the given name.
    pub fn named<N: Into<String>>(name: N) -> TargetBuilder {
        TargetBuilder::new(name)
    }

    /// Returns the width and height of the render target, measured in pixels.
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Returns the color buffer with index `i`.
    pub fn color_buf(&self, i: usize) -> Option<&ColorBuffer> {
        self.color_bufs.get(i)
    }

    /// Returns an array slice of the render target's color buffers.
    pub fn color_bufs(&self) -> &[ColorBuffer] {
        self.color_bufs.as_ref()
    }

    /// Returns the render target's depth-stencil buffer, if it has one.
    pub fn depth_buf(&self) -> Option<&DepthBuffer> {
        self.depth_buf.as_ref()
    }
}

impl<D> From<(Vec<ColorBuffer>, D, (u32, u32))> for Target
    where D: Into<Option<DepthBuffer>>
{
    fn from(data: (Vec<ColorBuffer>, D, (u32, u32))) -> Target {
        Target {
            color_bufs: data.0,
            depth_buf: data.1.into(),
            size: data.2,
        }
    }
}

/// Builds new render targets.
///
/// By default, it creates render targets with one color buffer and no
/// depth-stencil buffer.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TargetBuilder {
    custom_size: Option<(u32, u32)>,
    name: String,
    has_depth_buf: bool,
    num_color_bufs: usize,
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
    pub fn with_num_color_bufs(mut self, num: usize) -> Self {
        self.num_color_bufs = num;
        self
    }

    /// Specifies a custom target size.
    pub fn with_size(mut self, size: (u32, u32)) -> Self {
        self.custom_size = Some(size);
        self
    }

    /// Builds and returns the new render target.
    #[doc(hidden)]
    pub(crate) fn finish(self, fac: &mut Factory, size: (u32, u32)) -> Result<(String, Arc<Target>)> {
        use gfx::Factory;

        let size = self.custom_size.unwrap_or(size);

        let color_bufs = (0..self.num_color_bufs)
            .into_iter()
            .map(|_| {
                let (w, h) = (size.0 as u16, size.1 as u16);
                let (_, res, rt) = fac.create_render_target(w, h)?;
                Ok(ColorBuffer {
                    as_input: Some(res),
                    as_output: rt,
                })
            })
            .collect::<Result<_>>()?;

        let depth_buf = if self.has_depth_buf {
            let (w, h) = (size.0 as u16, size.1 as u16);
            let (_, res, dt) = fac.create_depth_stencil(w, h)?;
            let depth = DepthBuffer {
                as_input: Some(res),
                as_output: dt,
            };
            Some(depth)
        } else {
            None
        };

        let target = Target {
            color_bufs: color_bufs,
            depth_buf: depth_buf,
            size: size,
        };

        Ok((self.name, Arc::new(target)))
    }
}

impl<T> From<(T, usize, bool, (u32, u32))> for TargetBuilder
    where T: Into<String>
{
    fn from(builder: (T, usize, bool, (u32, u32))) -> TargetBuilder {
        TargetBuilder::new(builder.0)
            .with_num_color_bufs(builder.1)
            .with_depth_buf(builder.2)
            .with_size(builder.3)
    }
}

/// A hash map containing named render targets.
pub type Targets = HashMap<String, Arc<Target>>;
