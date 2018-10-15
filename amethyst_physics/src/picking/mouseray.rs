use amethyst_core::cgmath::prelude::*;
use amethyst_core::cgmath::{Matrix4, Point3, Vector3, Vector4};
use amethyst_core::specs::prelude::*;
use amethyst_core::transform::Transform;
use amethyst_input::InputHandler;
use amethyst_renderer::ActiveCamera;
use amethyst_renderer::{Camera, ScreenDimensions};

use collision::Ray3;

// TODO: use the actual Ray type from `collision`
/// Resource which contains the ray in world space of the mouse from the active camera
pub struct MouseRay {
    origin: Point3<f32>,
    direction: Vector3<f32>,
}

impl MouseRay {
    pub fn ray(&self) -> Ray3<f32> {
        Ray3::new(self.origin, self.direction)
    }
}

impl Default for MouseRay {
    fn default() -> Self {
        MouseRay {
            origin: Point3::new(0., 0., 0.),
            direction: Vector3::new(1., 1., 1.),
        }
    }
}

pub struct MouseRaySys;

impl<'s> System<'s> for MouseRaySys {
    type SystemData = (
        Read<'s, InputHandler<String, String>>,
        ReadExpect<'s, ActiveCamera>,
        ReadExpect<'s, ScreenDimensions>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, Transform>,
        Write<'s, MouseRay>,
    );

    fn run(&mut self, (input, activecam, dims, camera, transform, mut mouseray): Self::SystemData) {
        match (
            input.mouse_position(),
            camera.get(activecam.entity),
            transform.get(activecam.entity),
        ) {
            (Some((x, y)), Some(camera), Some(transform)) => {
                // FIXME: wrong if camera is a child of another entity with a transform
                //        might be better to extract the translation from GlobalTransform
                mouseray.origin = Point3::from_vec(transform.translation);
                mouseray.direction = from_window_space(
                    (x as f32, y as f32),
                    (dims.width(), dims.height()),
                    camera.proj,
                    transform.matrix(),
                );
            }
            (None, _, _) => (), // XXX: case happens whenever the window loses focus
            (_, None, _) => warn!("unable to fetch active-camera entity"),
            (_, _, None) => warn!("unable to fetch active-camera transform"),
        }
    }
}

// TODO: optimize this by caching eg. the inverted projection matrix
/// Convert a 2D point in window space to a 3D vector in world space
fn from_window_space(
    (window_x, window_y): (f32, f32),
    (width, height): (f32, f32),
    proj: Matrix4<f32>,
    view: Matrix4<f32>,
) -> Vector3<f32> {
    // Window
    //   <0,0> in the top-left
    let mut v = Vector4 {
        x: window_x,
        y: window_y,
        z: 0.,
        w: 0.,
    };

    // NDC
    //   shift and stretch x and y to range [-1,1]
    //   flip y
    v.x = 2. * v.x / width - 1.;
    v.y = 2. * (height - v.y) / height - 1.;

    // Clip

    // Camera
    //   unproject
    //   overwrite z
    v = proj.invert().expect("invert projection matrix") * v;
    v.z = -1.; // XXX: in Camera forward is -z

    // TODO: try doing Z before proj.invert(), and then cache the combination of proj*view

    // World
    //   transform & normalize
    v = view * v; // XXX: why is this just the normal view matrix??
    v = v.normalize();

    Vector3::new(v.x, v.y, v.z)
}
