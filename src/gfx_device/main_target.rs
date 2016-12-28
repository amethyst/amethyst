extern crate gfx;
extern crate gfx_device_gl;
use renderer;

/// Variants of this enum hold `main_color` and `main_depth` render targets.
// TODO: Don't export this enum.
pub enum MainTargetInner {
    OpenGL {
        main_color: gfx::handle::RenderTargetView<gfx_device_gl::Resources, renderer::target::ColorFormat>,
        main_depth: gfx::handle::DepthStencilView<gfx_device_gl::Resources, renderer::target::DepthFormat>,
    },
    #[cfg(windows)]
    Direct3D {
        // stub
    },
    Null,
}

/// This struct represents the screen render target.
pub struct MainTarget {
    pub main_target_inner: MainTargetInner,
}

impl MainTarget {
    pub fn new(main_target_inner: MainTargetInner) -> MainTarget {
        MainTarget {
            main_target_inner: main_target_inner,
        }
    }
}
