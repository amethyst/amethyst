//! Render target used for storing 2D pixel representations of 3D scenes.

use fnv::FnvHashMap as HashMap;
use serde::{Deserialize, Serialize};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{
    error::Result,
    types::{DepthStencilView, Encoder, Factory, RenderTargetView, ShaderResourceView, Window},
};

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

/// A hash map containing named render targets.
pub type Targets = HashMap<String, Target>;

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
    /// Creates a new `Target` from a single color buffer and depth buffer pair.
    pub(crate) fn new(cb: ColorBuffer, db: DepthBuffer, size: (u32, u32)) -> Self {
        Target {
            color_bufs: vec![cb],
            depth_buf: Some(db),
            size,
        }
    }

    /// Creates a new TargetBuilder with the given name.
    pub fn named<N: Into<String>>(name: N) -> TargetBuilder {
        TargetBuilder::new(name)
    }

    /// Clears all color buffers to the given value.
    pub fn clear_color<V: Into<[f32; 4]>>(&self, enc: &mut Encoder, value: V) {
        #[cfg(feature = "profiler")]
        profile_scope!("render_target_clearcolor");
        let val = value.into();
        for buf in self.color_bufs.iter() {
            enc.clear(&buf.as_output, val);
        }
    }

    /// Clears the depth stencil buffer to the given value.
    pub fn clear_depth_stencil<V: Into<f32>>(&self, enc: &mut Encoder, value: V) {
        if let Some(ref buf) = self.depth_buf {
            let val = value.into();
            enc.clear_depth(&buf.as_output, val);
            enc.clear_stencil(&buf.as_output, val as u8);
        }
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

    /// Creates the Direct3D 11 backend.
    #[cfg(all(feature = "d3d11", target_os = "windows"))]
    pub fn resize_main_target(window: &Window) -> Result<(Device, Factory, Target)> {
        unimplemented!()
    }

    #[cfg(all(feature = "metal", target_os = "macos"))]
    pub fn resize_main_target(window: &Window) -> Result<(Device, Factory, Target)> {
        unimplemented!()
    }

    /// Creates the OpenGL backend.
    #[cfg(feature = "opengl")]
    pub fn resize_main_target(&mut self, window: &Window) {
        #[cfg(feature = "profiler")]
        profile_scope!("render_target_resizemaintarget");
        if let Some(depth_buf) = self.depth_buf.as_mut() {
            for color_buf in &mut self.color_bufs {
                gfx_window_glutin::update_views(
                    window,
                    &mut color_buf.as_output,
                    &mut depth_buf.as_output,
                );
            }
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
    pub(crate) fn build(self, fac: &mut Factory, size: (u32, u32)) -> Result<(String, Target)> {
        use gfx::Factory;

        #[cfg(feature = "profiler")]
        profile_scope!("render_target_build");

        let size = self.custom_size.unwrap_or(size);

        let color_bufs = (0..self.num_color_bufs)
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
            color_bufs,
            depth_buf,
            size,
        };

        Ok((self.name, target))
    }
}
