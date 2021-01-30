use amethyst::{
    core::{math::Vector3, transform::Transform},
    ecs::{component, Entity, ParallelRunnable, System, SystemBuilder},
    input::InputHandler,
    prelude::IntoQuery,
    renderer::{ActiveCamera, Camera},
};
pub(crate) struct CameraMovementSystem;

impl System for CameraMovementSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("CameraMovementSystem")
                .read_resource::<ActiveCamera>()
                .read_resource::<InputHandler>()
                .with_query(<(Entity, &mut Transform)>::query().filter(component::<Camera>()))
                .build(move |_commands, world, (active_camera, input), query| {
                    let x_move = input.axis_value("camera_x").unwrap();
                    let y_move = input.axis_value("camera_y").unwrap();
                    let z_move = input.axis_value("camera_z").unwrap();
                    let z_move_scale = input.axis_value("camera_scale").unwrap();

                    if x_move != 0.0 || y_move != 0.0 || z_move != 0.0 || z_move_scale != 0.0 {
                        // Find if the active camera exists
                        let camera_transform = active_camera
                            .entity
                            .as_ref()
                            .and_then(|active_camera| {
                                query.get_mut(world, *active_camera).ok().map(|(_, c)| c)
                            })
                            .or(None);

                        // Return active camera or fetch first available
                        let camera_transform = match camera_transform {
                            Some(e) => Some(e),
                            None => query.iter_mut(world).next().map(|(_, t)| t),
                        };

                        if let Some(camera_transform) = camera_transform {
                            camera_transform.prepend_translation_x(x_move * 5.0);
                            camera_transform.prepend_translation_y(y_move * 5.0);
                            camera_transform.prepend_translation_z(z_move);

                            let z_scale = 0.01 * z_move_scale;
                            let scale = camera_transform.scale();
                            let scale = Vector3::new(
                                (scale.x + z_scale).max(0.001),
                                (scale.y + z_scale).max(0.001),
                                (scale.z + z_scale).max(0.001),
                            );
                            camera_transform.set_scale(scale);
                        }
                    }
                }),
        )
    }
}
