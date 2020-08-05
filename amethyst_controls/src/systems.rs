use winit::{DeviceEvent, Event, Window, WindowEvent};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use amethyst_core::{
    ecs::*,
    math::{convert, Unit, Vector3},
    shrev::{EventChannel, ReaderId},
    timing::Time,
    transform::Transform,
};
use std::collections::HashMap;

use amethyst_input::{get_input_axis_simple, BindingTypes, InputHandler};

use crate::{
    components::{ArcBallControl, FlyControl},
    resources::{HideCursor, WindowFocus},
};

/// The system that manages the fly movement.
///
/// # Type parameters
///
/// * `T`: This are the keys the `InputHandler` is using for axes and actions. Often, this is a `StringBindings`.
pub fn build_fly_movement_system<T: BindingTypes>(
    speed: f32,
    horizontal_axis: Option<T::Axis>,
    vertical_axis: Option<T::Axis>,
    longitudinal_axis: Option<T::Axis>,
) -> impl Runnable {
    SystemBuilder::new("FlyMovementSystem")
        .read_resource::<Time>()
        .read_resource::<InputHandler<T>>()
        .with_query(<(&FlyControl, &mut Transform)>::query())
        .build(move |_commands, world, (time, input), controls| {
            #[cfg(feature = "profiler")]
            profile_scope!("fly_movement_system");

            let x = get_input_axis_simple(&horizontal_axis, &input);
            let y = get_input_axis_simple(&vertical_axis, &input);
            let z = get_input_axis_simple(&longitudinal_axis, &input);

            if let Some(dir) = Unit::try_new(Vector3::new(x, y, z), convert(1.0e-6)) {
                for (_, transform) in controls.iter_mut(world) {
                    let delta_sec = time.delta_seconds();
                    transform.append_translation_along(dir, delta_sec * speed);
                }
            }
        })
}

/// The system that manages the arc ball movement;
/// In essence, the system will align the camera with its target while keeping the distance to it
/// and while keeping the orientation of the camera.
///
/// To modify the orientation of the camera in accordance with the mouse input, please use the
/// `FreeRotationSystem`.
pub fn build_arc_ball_rotation_system() -> impl Runnable {
    SystemBuilder::new("ArcBallRotationSystem")
        .with_query(<&ArcBallControl>::query())
        .with_query(<(&ArcBallControl, &mut Transform)>::query())
        .read_component::<Transform>()
        .build(move |_commands, world, (), queries| {
            #[cfg(feature = "profiler")]
            profile_scope!("arc_ball_rotation_system");

            let targets: HashMap<Entity, Transform> = queries
                .0
                .iter(world)
                .map(|ctrl| {
                    match world
                        .entry_ref(ctrl.target)
                        .map(|e| e.into_component::<Transform>().ok())
                        .flatten()
                    {
                        Some(trans) => Some((ctrl.target, *trans)),
                        None => None,
                    }
                })
                .filter(|t| t.is_some())
                .map(|t| t.unwrap())
                .collect();

            for (control, transform) in queries.1.iter_mut(world) {
                let pos_vec = transform.rotation() * -Vector3::z() * control.distance;
                match targets.get(&control.target) {
                    Some(target_trans) => {
                        *transform.translation_mut() = target_trans.translation() - pos_vec;
                    }
                    None => continue,
                }
            }
        })
}

/// The system that manages the view rotation.
///
/// Controlled by the mouse.
/// Goes into an inactive state if the window is not focused (`WindowFocus` resource).
///
/// Can be manually disabled by making the mouse visible using the `HideCursor` resource:
/// `HideCursor.hide = false`
pub fn build_free_rotation_system(
    sensitivity_x: f32,
    sensitivity_y: f32,
    mut reader: ReaderId<Event>,
) -> impl Runnable {
    SystemBuilder::new("FreeRotationSystem")
        .read_resource::<EventChannel<Event>>()
        .read_resource::<WindowFocus>()
        .read_resource::<HideCursor>()
        .with_query(
            <&mut Transform>::query()
                .filter(component::<FlyControl>() | component::<ArcBallControl>()),
        )
        .build(move |_commands, world, (events, focus, hide), controls| {
            #[cfg(feature = "profiler")]
            profile_scope!("free_rotation_system");

            let focused = focus.is_focused;
            for event in events.read(&mut reader) {
                if focused && hide.hide {
                    if let Event::DeviceEvent { ref event, .. } = *event {
                        if let DeviceEvent::MouseMotion { delta: (x, y) } = *event {
                            for transform in controls.iter_mut(world) {
                                transform.append_rotation_x_axis(
                                    (-(y as f32) * sensitivity_y).to_radians(),
                                );
                                transform.prepend_rotation_y_axis(
                                    (-(x as f32) * sensitivity_x).to_radians(),
                                );
                            }
                        }
                    }
                }
            }
        })
}

/// Builds the mouse focus update System.
pub fn build_mouse_focus_update_system(mut reader: ReaderId<Event>) -> impl Runnable {
    SystemBuilder::new("MouseFocusUpdateSystem")
        .read_resource::<EventChannel<Event>>()
        .write_resource::<WindowFocus>()
        .build(move |_commands, _world, (events, focus), ()| {
            #[cfg(feature = "profiler")]
            profile_scope!("mouse_focus_update_system");

            for event in events.read(&mut reader) {
                if let Event::WindowEvent { ref event, .. } = *event {
                    if let WindowEvent::Focused(focused) = *event {
                        focus.is_focused = focused;
                    }
                }
            }
        })
}

/// System which hides the cursor when the window is focused.
/// Requires the usage MouseFocusUpdateSystem at the same time.
pub fn build_cursor_hide_system() -> impl Runnable {
    let mut is_hidden = true;

    SystemBuilder::new("CursorHideSystem")
        .read_resource::<HideCursor>()
        .read_resource::<WindowFocus>()
        .read_resource::<Window>()
        .build(move |_commands, _world, (hide, focus, window), ()| {
            #[cfg(feature = "profiler")]
            profile_scope!("cursor_hide_system");

            let should_be_hidden = focus.is_focused && hide.hide;
            if !is_hidden && should_be_hidden {
                if let Err(err) = window.grab_cursor(true) {
                    log::error!("Unable to grab the cursor. Error: {:?}", err);
                }
                window.hide_cursor(true);
                is_hidden = true;
            } else if is_hidden && !should_be_hidden {
                if let Err(err) = window.grab_cursor(false) {
                    log::error!("Unable to release the cursor. Error: {:?}", err);
                }
                window.hide_cursor(false);
                is_hidden = false;
            }
        })
}
