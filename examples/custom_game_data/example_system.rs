use super::DemoState;
use amethyst::{
    core::{
        math::{UnitQuaternion, Vector3},
        Time, Transform,
    },
    ecs::prelude::{Entity, Join, Read, ReadStorage, System, WriteExpect, WriteStorage},
    renderer::{Camera, Light},
    ui::{UiFinder, UiText},
    utils::fps_counter::FPSCounter,
};

#[derive(Default)]
pub struct ExampleSystem {
    fps_display: Option<Entity>,
}

impl<'a> System<'a> for ExampleSystem {
    type SystemData = (
        WriteStorage<'a, Light>,
        Read<'a, Time>,
        ReadStorage<'a, Camera>,
        WriteStorage<'a, Transform<f32>>,
        WriteExpect<'a, DemoState>,
        WriteStorage<'a, UiText>,
        Read<'a, FPSCounter>,
        UiFinder<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut lights, time, camera, mut transforms, mut state, mut ui_text, fps_counter, finder) =
            data;
        let light_angular_velocity = -1.0;
        let light_orbit_radius = 15.0;
        let light_z = 6.0;

        let camera_angular_velocity = 0.1;

        state.light_angle += light_angular_velocity * time.delta_seconds();
        state.camera_angle += camera_angular_velocity * time.delta_seconds();

        let delta_rot = UnitQuaternion::from_axis_angle(
            &Vector3::z_axis(),
            camera_angular_velocity * time.delta_seconds(),
        );
        for (_, transform) in (&camera, &mut transforms).join() {
            // Append the delta rotation to the current transform.
            *transform.isometry_mut() = delta_rot * transform.isometry();
        }

        for (point_light, transform) in
            (&mut lights, &mut transforms)
                .join()
                .filter_map(|(light, transform)| {
                    if let Light::Point(ref mut point_light) = *light {
                        Some((point_light, transform))
                    } else {
                        None
                    }
                })
        {
            transform.set_translation_xyz(
                light_orbit_radius * state.light_angle.cos(),
                light_orbit_radius * state.light_angle.sin(),
                light_z,
            );

            point_light.color = state.light_color.into();
        }

        if let None = self.fps_display {
            if let Some(fps_entity) = finder.find("fps_text") {
                self.fps_display = Some(fps_entity);
            }
        }
        if let Some(fps_entity) = self.fps_display {
            if let Some(fps_display) = ui_text.get_mut(fps_entity) {
                if time.frame_number() % 20 == 0 {
                    let fps = fps_counter.sampled_fps();
                    fps_display.text = format!("FPS: {:.*}", 2, fps);
                }
            }
        }
    }
}
