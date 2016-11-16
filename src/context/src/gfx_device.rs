macro_rules! unwind_gfx_device_inner_mut {
    ($variable:expr, $field1:ident, $expr_field:expr, $expr_null:expr) => {
        match $variable {
            GfxDeviceInner::OpenGL {
                ref mut $field1,
                ..
            } => $expr_field,
            #[cfg(windows)]
            GfxDeviceInner::Direct3D { } => unimplemented!(),
            GfxDeviceInner::Null => $expr_null,
        }
    };
}

extern crate amethyst_renderer;
extern crate glutin;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate gfx;

use self::amethyst_renderer::{Renderer, Frame};
use self::amethyst_renderer::{Layer, Scene, Target, Camera, Light};
use self::amethyst_renderer::target::{ColorFormat, DepthFormat, ColorBuffer};

use amethyst_config::Element;
use std::path::Path;

/// GfxDevice owns all resources related to graphics (e.g. amethyst_renderer::Renderer, gfx_device_gl::Device,
/// gfx_device_gl::Factory amethyst_renderer::Frame).
pub struct GfxDevice {
    gfx_device_inner: GfxDeviceInner,
}

pub enum GfxDeviceInner {
    OpenGL {
        window: glutin::Window,
        device: gfx_device_gl::Device,
        factory: gfx_device_gl::Factory,
        renderer: Renderer<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
        frame: Frame<gfx_device_gl::Resources>,
    },

    #[cfg(windows)]
    Direct3D {
        // stub
    },

    Null,
}

config!(
    /// Contains display config,
    /// it is required to create a `VideoContext`
    struct DisplayConfig {
        pub title: String = "Amethyst game".to_string(),
        pub fullscreen: bool = false,
        pub dimensions: Option<(u32, u32)> = None,
        pub min_dimensions: Option<(u32, u32)> = None,
        pub max_dimensions: Option<(u32, u32)> = None,
        pub vsync: bool = true,
        pub multisampling: u16 = 1,
        pub visibility: bool = true,
        pub backend: String = "Null".to_string(),
    }
);

impl GfxDeviceInner {
    pub fn new(display_config: DisplayConfig) -> GfxDeviceInner {
        match display_config.backend.clone().as_ref() {
            "OpenGL" => new_gl(&display_config),
            #[cfg(windows)]
            "Direct3D" => new_d3d(),
            _ => GfxDeviceInner::Null,
        }
    }
}

#[cfg(windows)]
fn new_d3d() -> (VideoContext, FactoryImpl) {
    unimplemented!();
}

fn new_gl(display_config: &DisplayConfig) -> GfxDeviceInner {
    let title = display_config.title.clone();
    let multisampling = display_config.multisampling.clone();
    let visibility = display_config.visibility.clone();

    let mut builder = glutin::WindowBuilder::new()
        .with_title(title)
        .with_multisampling(multisampling)
        .with_visibility(visibility);

    if let Some((w, h)) = display_config.dimensions {
        builder = builder.with_dimensions(w, h);
    }

    if let Some((w_min, h_min)) = display_config.min_dimensions {
        builder = builder.with_min_dimensions(w_min, h_min);
    }

    if let Some((w_max, h_max)) = display_config.max_dimensions {
        builder = builder.with_max_dimensions(w_max, h_max);
    }

    if display_config.vsync {
        builder = builder.with_vsync();
    }

    if display_config.fullscreen {
        let monitor = glutin::get_primary_monitor();
        builder = builder.with_fullscreen(monitor);
    }

    let (window, device, mut factory, main_color, main_depth) = gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);

    let combuf = factory.create_command_buffer();
    let mut renderer = Renderer::new(combuf);
    renderer.load_all(&mut factory);

    let mut frame = Frame::new();
    frame.targets.insert("main".into(),
                         Box::new(ColorBuffer {
                             color: main_color,
                             output_depth: main_depth,
                         }));

    let (w, h) = window.get_inner_size().unwrap();
    frame.targets.insert("gbuffer".into(),
                         Box::new(amethyst_renderer::target::GeometryBuffer::new(&mut factory, (w as u16, h as u16))));

    let gfx_device_inner = GfxDeviceInner::OpenGL {
        window: window,
        device: device,
        factory: factory,
        renderer: renderer,
        frame: frame,
    };

    gfx_device_inner
}

impl GfxDevice {
    /// Create a new `Renderer` from `DisplayConfig`.
    pub fn new(gfx_device_inner: GfxDeviceInner) -> GfxDevice {
        GfxDevice { gfx_device_inner: gfx_device_inner }
    }

    /// Set the rendering pipeline to be used.
    pub fn set_pipeline(&mut self, pipeline: Vec<Layer>) {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            frame,
            {
                frame.layers = pipeline;
            },
            ()
        )
    }

    /// Add a rendering `Target`.
    pub fn add_target(&mut self, target: Box<Target>, name: &str) {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            frame,
            {
                frame.targets.insert(name.into(), target);
            },
            ()
        )
    }
    /// Delete a rendering `Target`.
    pub fn delete_target(&mut self, name: &str) {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            frame,
            {
                frame.targets.remove(name.into());
            },
            ()
        )
    }

    /// Add an empty `Scene`.
    pub fn add_scene(&mut self, name: &str) {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            frame,
            {
                let scene = Scene::new();
                frame.scenes.insert(name.into(), scene);
            },
            ()
        )
    }
    /// Delete a `Scene`.
    pub fn delete_scene(&mut self, name: &str) {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            frame,
            {
                frame.scenes.remove(name);
            },
            ()
        )
    }

    /// Add a `Fragment` to the scene with name `scene_name`.
    /// Return the index of the added `Fragment`.
    pub fn add_fragment(&mut self, scene_name: &str, fragment: Fragment) -> Option<usize> {
        match self.gfx_device_inner {
            GfxDeviceInner::OpenGL { ref mut frame, .. } => {
                let scene = match frame.scenes.get_mut(scene_name.into()) {
                    Some(scene) => scene,
                    None => return None,
                };
                match fragment.fragment_impl {
                    FragmentImpl::OpenGL { fragment } => {
                        scene.fragments.push(fragment);
                        Some(scene.fragments.len() - 1)
                    }
                    #[cfg(windows)]
                    FragmentImpl::Direct3D {} => unimplemented!(),
                    FragmentImpl::Null => None,
                }
            }
            #[cfg(windows)]
            GfxDeviceInner::Direct3D {} => {
                unimplemented!();
            }
            GfxDeviceInner::Null => None,
        }
    }
    /// Get a mutable reference to the transform field of `Fragment` with index `idx`
    /// in scene `scene_name`.
    pub fn mut_fragment_transform(&mut self, scene_name: &str, idx: usize) -> Option<&mut [[f32; 4]; 4]> {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            frame,
            {
                let scene = match frame.scenes.get_mut(scene_name.into()) {
                    Some(scene) => scene,
                    None => return None,
                };
                Some(&mut scene.fragments[idx].transform)
            },
            None
        )
    }
    /// Delete `Fragment` with index `idx` in scene `scene_name`.
    pub fn delete_fragment(&mut self, scene_name: &str, idx: usize) {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            frame,
            {
                let scene = match frame.scenes.get_mut(scene_name.into()) {
                    Some(scene) => scene,
                    None => return,
                };
                scene.fragments.remove(idx);
            },
            ()
        )
    }

    // Return number of fragments in scene `scene_name`.
    pub fn num_fragments(&mut self, scene_name: &str) -> Option<usize> {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            frame,
            {
                let scene = match frame.scenes.get_mut(scene_name.into()) {
                    Some(scene) => scene,
                    None => return None,
                };
                Some(scene.fragments.len())
            },
            None
        )
    }

    /// Add a `Light` to the scene `scene_name`.
    /// Return the index of the added `Light`.
    pub fn add_light(&mut self, scene_name: &str, light: Light) -> Option<usize> {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            frame,
            {
                let scene = match frame.scenes.get_mut(scene_name.into()) {
                    Some(scene) => scene,
                    None => return None,
                };
                scene.lights.push(light);
                Some(scene.lights.len() - 1)
            },
            None
        )
    }
    /// Lookup `Light` in scene `scene_name` by index.
    pub fn mut_light(&mut self, scene_name: &str, idx: usize) -> Option<&mut Light> {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            frame,
            {
                let scene = match frame.scenes.get_mut(scene_name.into()) {
                    Some(scene) => scene,
                    None => return None,
                };
                scene.lights.get_mut(idx)
            },
            None
        )
    }
    /// Delete `Light` with index `idx` in scene `scene_name`.
    pub fn delete_light(&mut self, scene_name: &str, idx: usize) {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            frame,
            {
                let scene = match frame.scenes.get_mut(scene_name.into()) {
                    Some(scene) => scene,
                    None => return,
                };
                scene.lights.remove(idx);
            },
            ()
        )
    }

    // Return number of lights in scene `scene_name`.
    pub fn num_lights(&mut self, scene_name: &str) -> Option<usize> {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            frame,
            {
                let scene = match frame.scenes.get_mut(scene_name.into()) {
                    Some(scene) => scene,
                    None => return None,
                };
                Some(scene.lights.len())
            },
            None
        )
    }

    /// Add a `Camera`.
    pub fn add_camera(&mut self, camera: Camera, name: &str) {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            frame,
            {
                frame.cameras.insert(name.into(), camera);
            },
            ()
        )
    }
    /// Lookup a `Camera` by name.
    pub fn mut_camera(&mut self, name: &str) -> Option<&mut Camera> {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            frame,
            {
                frame.cameras.get_mut(name.into())
            },
            None
        )
    }
    /// Delete a `Camera`.
    pub fn delete_camera(&mut self, name: &str) {
        unwind_gfx_device_inner_mut!(
            self.gfx_device_inner,
            frame,
            {
                frame.cameras.remove(name.into());
            },
            ()
        )
    }

    pub fn get_dimensions(&self) -> Option<(u32, u32)> {
        match self.gfx_device_inner {
            GfxDeviceInner::OpenGL { ref window, .. } => window.get_inner_size(),
            #[cfg(windows)]
            GfxDeviceInner::Direct3D {} => unimplemented!(),
            GfxDeviceInner::Null => None,
        }
    }

    /// Submit the `Frame` to `amethyst_renderer::Renderer`.
    pub fn submit(&mut self) {
        match self.gfx_device_inner {
            GfxDeviceInner::OpenGL { ref window,
                                     ref mut device,
                                     ref mut renderer,
                                     ref frame,
                                     .. } => {
                renderer.submit(frame, device);
                window.swap_buffers().unwrap();
            }
            #[cfg(windows)]
            GfxDeviceInner::Direct3D {} => unimplemented!(),
            GfxDeviceInner::Null => (),
        }
    }
}

/// An enum with variants representing concrete
/// `Fragment` types compatible with different backends.
pub enum FragmentImpl {
    OpenGL { fragment: amethyst_renderer::Fragment<gfx_device_gl::Resources>, },
    #[cfg(windows)]
    Direct3D {
        // stub
    },
    Null,
}

/// A wraper around `Fragment` required to
/// hide all platform specific code from the user.
pub struct Fragment {
    pub fragment_impl: FragmentImpl,
}
