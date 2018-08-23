use amethyst::controls::XYControlTag;
use amethyst::core::{GlobalTransform, Transform};
use amethyst::ecs::World;
use amethyst::prelude::*;
use amethyst::renderer::{Camera, Projection};

const CAMERA_EDGE: f32 = 512.0;
const CAMERA_SPEED: f32 = 512.0;
const CAMERA_ZOOM_SPEED: f32 = 0.05;
const VERTICAL_BORDERS: (f32, f32) = (-300.0, 400.0);
const HORIZONTAL_BORDERS: (f32, f32) = (150.0, 1900.0);
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
        .with(
            XYControlTag::new(CAMERA_SPEED, CAMERA_SPEED, CAMERA_ZOOM_SPEED)
                .with_horizontal_borders(HORIZONTAL_BORDERS)
                .with_vertical_borders(VERTICAL_BORDERS)
                .with_zoom_borders(ZOOM_BORDERS),
        )
        .with(GlobalTransform::default())
        .with(transform)
        .build();
}
