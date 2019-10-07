use derive_new::new;
use winit::{DeviceEvent, Event, Window, WindowEvent};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use amethyst_core::{
    ecs::prelude::{
        Join, Read, ReadExpect, ReadStorage, System, SystemData, World, Write, WriteStorage,
    },
    math::{convert, Unit, Vector3},
    shrev::{EventChannel, ReaderId},
    timing::Time,
    transform::Transform,
    SystemDesc,
};
use amethyst_derive::SystemDesc;
use amethyst_input::{get_input_axis_simple, BindingTypes, InputHandler};

use crate::{
    components::{ArcBallControlTag, FlyControlTag},
    resources::{HideCursor, WindowFocus},
};

/// The system that manages the fly movement.
///
/// # Type parameters
///
/// * `T`: This are the keys the `InputHandler` is using for axes and actions. Often, this is a `StringBindings`.
#[derive(Debug, SystemDesc)]
#[system_desc(name(FlyMovementSystemDesc))]
pub struct FlyMovementSystem<T>
where
    T: BindingTypes,
{
    /// The movement speed of the movement in units per second.
    speed: f32,
    /// The name of the input axis to locally move in the x coordinates.
    right_input_axis: Option<T::Axis>,
    /// The name of the input axis to locally move in the y coordinates.
    up_input_axis: Option<T::Axis>,
    /// The name of the input axis to locally move in the z coordinates.
    forward_input_axis: Option<T::Axis>,
}

impl<T: BindingTypes> FlyMovementSystem<T> {
    /// Builds a new `FlyMovementSystem` using the provided speeds and axis controls.
    pub fn new(
        speed: f32,
        right_input_axis: Option<T::Axis>,
        up_input_axis: Option<T::Axis>,
        forward_input_axis: Option<T::Axis>,
    ) -> Self {
        FlyMovementSystem {
            speed,
            right_input_axis,
            up_input_axis,
            forward_input_axis,
        }
    }
}

impl<'a, T: BindingTypes> System<'a> for FlyMovementSystem<T> {
    type SystemData = (
        Read<'a, Time>,
        WriteStorage<'a, Transform>,
        Read<'a, InputHandler<T>>,
        ReadStorage<'a, FlyControlTag>,
    );

    fn run(&mut self, (time, mut transform, input, tag): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("fly_movement_system");

        let x = get_input_axis_simple(&self.right_input_axis, &input);
        let y = get_input_axis_simple(&self.up_input_axis, &input);
        let z = get_input_axis_simple(&self.forward_input_axis, &input);

        if let Some(dir) = Unit::try_new(Vector3::new(x, y, z), convert(1.0e-6)) {
            for (transform, _) in (&mut transform, &tag).join() {
                let delta_sec = time.delta_seconds();
                transform.append_translation_along(dir, delta_sec * self.speed);
            }
        }
    }
}

/// The system that manages the arc ball movement;
/// In essence, the system will align the camera with its target while keeping the distance to it
/// and while keeping the orientation of the camera.
///
/// To modify the orientation of the camera in accordance with the mouse input, please use the
/// `FreeRotationSystem`.
#[derive(Debug, Default)]
pub struct ArcBallRotationSystem;

impl<'a> System<'a> for ArcBallRotationSystem {
    type SystemData = (
        WriteStorage<'a, Transform>,
        ReadStorage<'a, ArcBallControlTag>,
    );

    fn run(&mut self, (mut transforms, tags): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("arc_ball_rotation_system");

        let mut position = None;
        for (transform, arc_ball_camera_tag) in (&transforms, &tags).join() {
            let pos_vec = transform.rotation() * -Vector3::z() * arc_ball_camera_tag.distance;
            if let Some(target_transform) = transforms.get(arc_ball_camera_tag.target) {
                position = Some(target_transform.translation() - pos_vec);
            }
        }
        if let Some(new_pos) = position {
            for (transform, _) in (&mut transforms, &tags).join() {
                *transform.translation_mut() = new_pos;
            }
        }
    }
}

/// The system that manages the view rotation.
///
/// Controlled by the mouse.
/// Goes into an inactive state if the window is not focused (`WindowFocus` resource).
///
/// Can be manually disabled by making the mouse visible using the `HideCursor` resource:
/// `HideCursor.hide = false`
#[derive(Debug, SystemDesc, new)]
#[system_desc(name(FreeRotationSystemDesc))]
pub struct FreeRotationSystem {
    sensitivity_x: f32,
    sensitivity_y: f32,
    #[system_desc(event_channel_reader)]
    event_reader: ReaderId<Event>,
}

impl<'a> System<'a> for FreeRotationSystem {
    type SystemData = (
        Read<'a, EventChannel<Event>>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, FlyControlTag>,
        Read<'a, WindowFocus>,
        Read<'a, HideCursor>,
    );

    fn run(&mut self, (events, mut transform, tag, focus, hide): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("free_rotation_system");

        let focused = focus.is_focused;
        for event in events.read(&mut self.event_reader) {
            if focused && hide.hide {
                if let Event::DeviceEvent { ref event, .. } = *event {
                    if let DeviceEvent::MouseMotion { delta: (x, y) } = *event {
                        for (transform, _) in (&mut transform, &tag).join() {
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
    }
}

/// A system which reads Events and saves if a window has lost focus in a WindowFocus resource
#[derive(Debug, SystemDesc, new)]
#[system_desc(name(MouseFocusUpdateSystemDesc))]
pub struct MouseFocusUpdateSystem {
    #[system_desc(event_channel_reader)]
    event_reader: ReaderId<Event>,
}

impl<'a> System<'a> for MouseFocusUpdateSystem {
    type SystemData = (Read<'a, EventChannel<Event>>, Write<'a, WindowFocus>);

    fn run(&mut self, (events, mut focus): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("mouse_focus_update_system");

        for event in events.read(&mut self.event_reader) {
            if let Event::WindowEvent { ref event, .. } = *event {
                if let WindowEvent::Focused(focused) = *event {
                    focus.is_focused = focused;
                }
            }
        }
    }
}

/// System which hides the cursor when the window is focused.
/// Requires the usage MouseFocusUpdateSystem at the same time.
#[derive(Debug, SystemDesc, new)]
#[system_desc(name(CursorHideSystemDesc))]
pub struct CursorHideSystem {
    #[new(value = "true")]
    #[system_desc(skip)]
    is_hidden: bool,
}

impl Default for CursorHideSystem {
    fn default() -> Self {
        CursorHideSystem::new()
    }
}

impl<'a> System<'a> for CursorHideSystem {
    type SystemData = (
        ReadExpect<'a, Window>,
        Read<'a, HideCursor>,
        Read<'a, WindowFocus>,
    );

    fn run(&mut self, (win, hide, focus): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("cursor_hide_system");

        let should_be_hidden = focus.is_focused && hide.hide;
        if !self.is_hidden && should_be_hidden {
            if let Err(err) = win.grab_cursor(true) {
                log::error!("Unable to grab the cursor. Error: {:?}", err);
            }
            win.hide_cursor(true);
            self.is_hidden = true;
        } else if self.is_hidden && !should_be_hidden {
            if let Err(err) = win.grab_cursor(false) {
                log::error!("Unable to release the cursor. Error: {:?}", err);
            }
            win.hide_cursor(false);
            self.is_hidden = false;
        }
    }
}
