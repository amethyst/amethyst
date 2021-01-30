use amethyst::{
    core::{
        math::{Point3, Vector2},
        transform::Transform,
    },
    ecs::{Entity, IntoQuery, ParallelRunnable, System, SystemBuilder},
    input::InputHandler,
    renderer::{debug_drawing::DebugLinesComponent, palette::Srgba, ActiveCamera, Camera},
    window::ScreenDimensions,
};
#[derive(Default)]
pub struct DrawSelectionSystem {
    start_coordinate: Option<Point3<f32>>,
}

impl System for DrawSelectionSystem {
    fn build(mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("DrawSelectionSystem")
                .write_resource::<ActiveCamera>()
                .read_resource::<ScreenDimensions>()
                .read_resource::<InputHandler>()
                .with_query(<&mut DebugLinesComponent>::query())
                .with_query(<(Entity, &Camera, &Transform)>::query())
                .build(
                    move |_commands,
                          world,
                          (active_camera, dimensions, input),
                          (debug_lines_query, camera_query)| {
                        let (mut debug_lines_subworld, mut subworld) =
                            world.split_for_query(&debug_lines_query);
                        if let Some(lines) =
                            debug_lines_query.iter_mut(&mut debug_lines_subworld).next()
                        {
                            lines.clear();

                            if let Some(mouse_position) = input.mouse_position() {
                                // Find if the active camera exists
                                let camera_transform = active_camera
                                    .entity
                                    .as_ref()
                                    .and_then(|active_camera| {
                                        camera_query
                                            .get_mut(&mut subworld, *active_camera)
                                            .ok()
                                            .map(|(_, c, t)| (c, t))
                                    })
                                    .or(None);

                                // Return active camera or fetch first available
                                let camera_transform = match camera_transform {
                                    Some(e) => Some(e),
                                    None => {
                                        camera_query
                                            .iter_mut(&mut subworld)
                                            .next()
                                            .map(|(_, c, t)| (c, t))
                                    }
                                };

                                if let Some((camera, camera_transform)) = camera_transform {
                                    let action_down = input
                                        .action_is_down("select")
                                        .expect("selection action missing");
                                    if action_down && self.start_coordinate.is_none() {
                                        // Starting a new selection
                                        self.start_coordinate = Some(Point3::new(
                                            mouse_position.0,
                                            mouse_position.1,
                                            camera_transform.translation().z,
                                        ));
                                    } else if action_down && self.start_coordinate.is_some() {
                                        // Active drag
                                        let screen_dimensions =
                                            Vector2::new(dimensions.width(), dimensions.height());
                                        let end_coordinate = Point3::new(
                                            mouse_position.0,
                                            mouse_position.1,
                                            camera_transform.translation().z,
                                        );

                                        let mut start_world = camera.screen_to_world_point(
                                            self.start_coordinate.expect("Wut?"),
                                            screen_dimensions,
                                            camera_transform,
                                        );
                                        let mut end_world = camera.screen_to_world_point(
                                            end_coordinate,
                                            screen_dimensions,
                                            camera_transform,
                                        );
                                        start_world.z = 0.9;
                                        end_world.z = 0.9;

                                        lines.add_box(
                                            start_world,
                                            end_world,
                                            Srgba::new(0.5, 0.05, 0.65, 1.0),
                                        );
                                    } else if !action_down && self.start_coordinate.is_some() {
                                        // End drag, remove
                                        self.start_coordinate = None;
                                    }
                                }
                            }
                        }
                    },
                ),
        )
    }
}
