use amethyst::core::cgmath::Ortho;
use amethyst::core::{GlobalTransform, Time, Transform};
use amethyst::ecs::{Entities, Join, Read, ReadExpect, ReadStorage, System, World, WriteStorage};
use amethyst::input::InputHandler;
use amethyst::prelude::*;
use amethyst::renderer::{ActiveCamera, Camera, Projection};

const CAMERA_EDGE: f32 = 512.0;
const CAMERA_SPEED: f32 = 512.0;
const CAMERA_ZOOM_SPEED: f32 = 0.05;
const VERTICAL_BORDERS: (f32, f32) = (-300.0, 400.0);
const HORZONTAL_BORDERS: (f32, f32) = (150.0, 1900.0);
const ZOOM_BORDERS: (f32, f32) = (0.4, 2.5);

pub fn initialise_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.translation.x = 300.0;
    transform.translation.y = 0.0;

    // Even though this is 2D, we move the camera very high in Z as
    // the draw order for sprites is determined by the Z value.
    // As those ordering values can get very high, if we don't
    // move the camera high enough, the sprites will effectively
    // be behind the camera and so wouldn't render.
    // This is an implementation detail that should be made
    // transparent in the future. Note that this does not
    // change at all the size of the sprites displayed, it
    // really has no effect besides making sure all sprites
    // are rendered.
    transform.translation.z = 500.0;

    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            -CAMERA_EDGE * 0.5,
            CAMERA_EDGE * 0.5,
            CAMERA_EDGE * 0.5,
            -CAMERA_EDGE * 0.5,
        )))
        .with(GlobalTransform::default())
        .with(transform)
        .build();
}

pub struct MoveCamera;

impl<'a> System<'a> for MoveCamera {
    type SystemData = (
        ReadStorage<'a, Camera>,
        Entities<'a>,
        Read<'a, Time>,
        Read<'a, InputHandler<String, String>>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (cameras, entities, time, input, mut transf): Self::SystemData) {
        if let Some((camera, _)) = (&*entities, &cameras).join().next() {
            if let Some(camera_transf) = transf.get_mut(camera) {
                if let Some(horiz) = input.axis_value("horizontal") {
                    camera_transf.translation.x = (camera_transf.translation.x
                        + CAMERA_SPEED * time.delta_seconds() * horiz as f32)
                        .max(HORZONTAL_BORDERS.0)
                        .min(HORZONTAL_BORDERS.1);
                }
                if let Some(vert) = input.axis_value("vertical") {
                    camera_transf.translation.y = (camera_transf.translation.y
                        + CAMERA_SPEED * time.delta_seconds() * vert as f32)
                        .max(VERTICAL_BORDERS.0)
                        .min(VERTICAL_BORDERS.1);
                }
                if let Some(zoom) = input.axis_value("zoom") {
                    camera_transf.scale.x = (camera_transf.scale.x
                        + CAMERA_ZOOM_SPEED * zoom as f32)
                        .max(ZOOM_BORDERS.0)
                        .min(ZOOM_BORDERS.1);
                    camera_transf.scale.y = (camera_transf.scale.y
                        + CAMERA_ZOOM_SPEED * zoom as f32)
                        .max(ZOOM_BORDERS.0)
                        .min(ZOOM_BORDERS.1);
                }
            }
        }
    }
}
