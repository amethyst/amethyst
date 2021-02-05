use amethyst::{
    core::{
        math::Vector3,
        transform::{Parent, Transform},
        Named,
    },
    ecs::{component, Entity, IntoQuery, ParallelRunnable, System, SystemBuilder},
    input::InputHandler,
    renderer::{ActiveCamera, Camera},
    window::ScreenDimensions,
};

pub struct CameraSwitchSystem {
    pressed: bool,
    perspective: bool,
}
impl Default for CameraSwitchSystem {
    fn default() -> Self {
        Self {
            pressed: false,
            perspective: false,
        }
    }
}
impl System for CameraSwitchSystem {
    fn build(mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("CameraSwitchSystem")
                .write_resource::<ActiveCamera>()
                .read_resource::<ScreenDimensions>()
                .read_resource::<InputHandler>()
                .with_query(
                    <(Entity, &Parent)>::query()
                        .filter(component::<Camera>() & component::<Transform>()),
                )
                .build(
                    move |commands, world, (active_camera, dimensions, input), camera_query| {
                        if input.action_is_down("camera_switch").unwrap() {
                            self.pressed = true;
                        }
                        if self.pressed && !input.action_is_down("camera_switch").unwrap() {
                            self.pressed = false;

                            // Lazily add new camera and delete the old camera
                            let entity_and_parent = active_camera
                                .entity
                                .and_then(|e| camera_query.get(world, e).ok())
                                .or_else(|| camera_query.iter(world).next());

                            if let Some((old_camera_entity, old_parent)) = entity_and_parent {
                                let old_camera_entity = old_camera_entity;

                                let new_parent = old_parent.0;

                                self.perspective = !self.perspective;
                                let (new_camera, new_position) = if self.perspective {
                                    (
                                        Camera::standard_3d(
                                            dimensions.width(),
                                            dimensions.height(),
                                        ),
                                        Vector3::new(0.0, 0.0, 500.1),
                                    )
                                } else {
                                    (
                                        Camera::standard_2d(
                                            dimensions.width(),
                                            dimensions.height(),
                                        ),
                                        Vector3::new(0.0, 0.0, 1.1),
                                    )
                                };

                                let new_camera = commands.push((
                                    Named("camera".into()),
                                    Parent(new_parent),
                                    Transform::from(new_position),
                                    new_camera,
                                ));
                                active_camera.entity = Some(new_camera);
                                commands.remove(*old_camera_entity);
                            }
                        }
                    },
                ),
        )
    }
}
