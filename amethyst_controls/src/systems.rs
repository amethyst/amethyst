use std::{borrow::Cow, collections::HashMap};

use amethyst_core::{
    dispatcher::ThreadLocalSystem,
    ecs::*,
    math::{convert, Unit, Vector3},
    shrev::{EventChannel, ReaderId},
    transform::Transform,
    Time,
};
use amethyst_input::{get_input_axis_simple, InputHandler};
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    window::Window,
};

use crate::{
    components::{ArcBallControl, FlyControl},
    resources::{HideCursor, WindowFocus},
};

/// The system that manages the fly movement.
#[derive(Debug)]
pub struct FlyMovementSystem {
    pub(crate) speed: f32,
    pub(crate) horizontal_axis: Option<Cow<'static, str>>,
    pub(crate) vertical_axis: Option<Cow<'static, str>>,
    pub(crate) longitudinal_axis: Option<Cow<'static, str>>,
}

impl System for FlyMovementSystem {
    fn build(self) -> Box<dyn systems::ParallelRunnable> {
        Box::new(
            SystemBuilder::new("FlyMovementSystem")
                .read_resource::<Time>()
                .read_resource::<InputHandler>()
                .with_query(<(&FlyControl, &mut Transform)>::query())
                .build(move |_commands, world, (time, input), controls| {
                    #[cfg(feature = "profiler")]
                    profile_scope!("fly_movement_system");

                    let x = get_input_axis_simple(&self.horizontal_axis, &input);
                    let y = get_input_axis_simple(&self.vertical_axis, &input);
                    let z = get_input_axis_simple(&self.longitudinal_axis, &input);

                    if let Some(dir) = Unit::try_new(Vector3::new(x, y, z), convert(1.0e-6)) {
                        for (_, transform) in controls.iter_mut(world) {
                            let delta_sec = time.delta_time().as_secs_f32();
                            transform.append_translation_along(dir, delta_sec * self.speed);
                        }
                    }
                }),
        )
    }
}

/// The system that manages the arc ball movement;
/// In essence, the system will align the camera with its target while keeping the distance to it
/// and while keeping the orientation of the camera.
///
/// To modify the orientation of the camera in accordance with the mouse input, please use the
/// `FreeRotationSystem`.
#[derive(Debug)]
pub struct ArcBallRotationSystem;

impl System for ArcBallRotationSystem {
    fn build(self) -> Box<dyn systems::ParallelRunnable> {
        Box::new(
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
                                .ok()
                                .and_then(|e| e.into_component::<Transform>().ok())
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
                }),
        )
    }
}

/// The system that manages the view rotation.
///
/// Controlled by the mouse.
/// Goes into an inactive state if the window is not focused (`WindowFocus` resource).
///
/// Can be manually disabled by making the mouse visible using the `HideCursor` resource:
/// `HideCursor.hide = false`
#[derive(Debug)]
pub struct FreeRotationSystem {
    pub(crate) sensitivity_x: f32,
    pub(crate) sensitivity_y: f32,
    pub(crate) reader: ReaderId<Event<'static, ()>>,
}

impl System for FreeRotationSystem {
    fn build(mut self) -> Box<dyn systems::ParallelRunnable> {
        Box::new(
            SystemBuilder::new("FreeRotationSystem")
                .read_resource::<EventChannel<Event<'static, ()>>>()
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
                    for event in events.read(&mut self.reader) {
                        if focused && hide.hide {
                            if let Event::DeviceEvent { ref event, .. } = *event {
                                if let DeviceEvent::MouseMotion { delta: (x, y) } = *event {
                                    for transform in controls.iter_mut(world) {
                                        transform.append_rotation_x_axis(
                                            (-(y as f32) * self.sensitivity_y).to_radians(),
                                        );
                                        transform.prepend_rotation_y_axis(
                                            (-(x as f32) * self.sensitivity_x).to_radians(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }),
        )
    }
}

/// Reports the status of window focus.
#[derive(Debug)]
pub struct MouseFocusUpdateSystem {
    // reads WindowEvent from winit
    pub(crate) reader: ReaderId<Event<'static, ()>>,
}

impl System for MouseFocusUpdateSystem {
    fn build(mut self) -> Box<dyn systems::ParallelRunnable> {
        Box::new(
            SystemBuilder::new("MouseFocusUpdateSystem")
                .read_resource::<EventChannel<Event<'static, ()>>>()
                .write_resource::<WindowFocus>()
                .build(move |_commands, _world, (events, focus), ()| {
                    #[cfg(feature = "profiler")]
                    profile_scope!("mouse_focus_update_system");

                    for event in events.read(&mut self.reader) {
                        if let Event::WindowEvent { ref event, .. } = *event {
                            if let WindowEvent::Focused(focused) = *event {
                                log::debug!("Window was focused.");
                                focus.is_focused = focused;
                            }
                        }
                    }
                }),
        )
    }
}

/// System which hides the cursor when the window is focused.
/// Requires the usage MouseFocusUpdateSystem at the same time.
#[derive(Debug)]
pub struct CursorHideSystem;

impl ThreadLocalSystem<'_> for CursorHideSystem {
    fn build(self) -> Box<dyn systems::Runnable> {
        let mut is_hidden = false;

        Box::new(
            SystemBuilder::new("CursorHideSystem")
                .read_resource::<HideCursor>()
                .read_resource::<WindowFocus>()
                .read_resource::<Window>()
                .build(move |_commands, _world, (hide, focus, window), ()| {
                    #[cfg(feature = "profiler")]
                    profile_scope!("cursor_hide_system");

                    let should_be_hidden = focus.is_focused && hide.hide;

                    if should_be_hidden != is_hidden {
                        window
                            .set_cursor_grab(should_be_hidden)
                            .expect("Failed to set cursor grab state.");
                        window.set_cursor_visible(!should_be_hidden);
                        is_hidden = should_be_hidden;
                    }
                }),
        )
    }
}
