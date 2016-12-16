extern crate gfx_device_gl;

/// This loader can be added to `AssetManager` to allow loading of `Mesh`es and `Texture`s.
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
