extern crate gfx_device_gl;

pub enum GfxLoader {
    OpenGL {
        factory: gfx_device_gl::Factory,
    },
    #[cfg(windows)]
    Direct3D {
        // stub
    },
    Null,
}
