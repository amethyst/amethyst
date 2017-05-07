//! Clears the color and/or depth buffers in a target.

use pipe::pass::PassBuilder;

/// Clears the color and/or depth buffers in a target.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ClearTarget {
    color_val: Option<[f32; 4]>,
    depth_val: Option<f32>,
}

impl ClearTarget {
    /// Creates a new ClearTarget pass with the given values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use amethyst_renderer::pass::ClearTarget;
    /// #
    /// // Clears color buffers to red, clears depth buffer to 0.0.
    /// ClearTarget::with_values([1.0, 0.0, 0.0, 1.0], 0.0);
    /// // Clears color buffers to transparent black, ignores depth buffer.
    /// ClearTarget::with_values([0.0; 4], None);
    /// // Ignores color buffers, clears the depth buffer to 0.5.
    /// ClearTarget::with_values(None, 0.5);
    /// ```
    pub fn with_values<C, D>(color_val: C, depth_val: D) -> ClearTarget
        where C: Into<Option<[f32; 4]>>,
              D: Into<Option<f32>>
    {
        ClearTarget {
            color_val: color_val.into(),
            depth_val: depth_val.into(),
        }
    }
}

impl Into<PassBuilder> for ClearTarget {
    fn into(self) -> PassBuilder {
        PassBuilder::function(move |ref mut enc, ref out| {
            if let Some(val) = self.color_val {
                for buf in out.color_bufs() {
                    enc.clear(&buf.as_output, val);
                }
            }

            if let Some(val) = self.depth_val {
                if let Some(buf) = out.depth_buf() {
                    enc.clear_depth(&buf.as_output, val);
                    enc.clear_stencil(&buf.as_output, val as u8);
                }
            }
        })
    }
}
