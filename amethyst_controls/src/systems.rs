use crate::{
    components::{ArcBallControlTagComponent, FlyControlTagComponent},
    resources::{HideCursor, WindowFocus},
};
use amethyst_core::{
    ecs::prelude::{Join, Read, ReadExpect, ReadStorage, Resources, System, Write, WriteStorage},
    math::{convert, Unit, Vector3},
    shrev::{EventChannel, ReaderId},
    timing::Time,
    transform::TransformComponent,
    Float,
};
use amethyst_input::{get_input_axis_simple, BindingTypes, InputHandler};
use std::sync::Arc;
use winit::{DeviceEvent, Event, Window, WindowEvent};

/// The system that manages the fly movement.
///
/// # Type parameters
///
/// * `T`: This are the keys the `InputHandler` is using for axes and actions. Often, this is a `StringBindings`.
pub struct FlyMovementSystem<T: BindingTypes> {
    /// The movement speed of the movement in units per second.
    speed: Float,
    /// The name of the input axis to locally move in the x coordinates.
    right_input_axis: Option<T::Axis>,
    /// The name of the input axis to locally move in the y coordinates.
    up_input_axis: Option<T::Axis>,
    /// The name of the input axis to locally move in the z coordinates.
    forward_input_axis: Option<T::Axis>,
}

impl<T: BindingTypes> FlyMovementSystem<T> {
    /// Builds a new `FlyMovementSystem` using the provided speeds and axis controls.
    pub fn new<N: Into<Float>>(
        speed: N,
        right_input_axis: Option<T::Axis>,
        up_input_axis: Option<T::Axis>,
        forward_input_axis: Option<T::Axis>,
    ) -> Self {
        FlyMovementSystem {
            speed: speed.into(),
            right_input_axis,
            up_input_axis,
            forward_input_axis,
        }
    }
}

impl<'a, T: BindingTypes> System<'a> for FlyMovementSystem<T> {
    type SystemData = (
        Read<'a, Time>,
        WriteStorage<'a, TransformComponent>,
        Read<'a, InputHandler<T>>,
        ReadStorage<'a, FlyControlTagComponent>,
    );

    fn run(&mut self, (time, mut transform, input, tag): Self::SystemData) {
        let x: Float = get_input_axis_simple(&self.right_input_axis, &input).into();
        let y: Float = get_input_axis_simple(&self.up_input_axis, &input).into();
        let z: Float = get_input_axis_simple(&self.forward_input_axis, &input).into();

        if let Some(dir) = Unit::try_new(Vector3::new(x, y, z), convert(1.0e-6)) {
            for (transform, _) in (&mut transform, &tag).join() {
                let delta_sec = time.delta_seconds() as f64;
                transform.append_translation_along(dir, delta_sec * self.speed.as_f64());
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
///
pub struct ArcBallRotationSystem;

impl Default for ArcBallRotationSystem {
    fn default() -> Self {
        Self
    }
}

impl<'a> System<'a> for ArcBallRotationSystem {
    type SystemData = (
        WriteStorage<'a, TransformComponent>,
        ReadStorage<'a, ArcBallControlTagComponent>,
    );

    fn run(&mut self, (mut transforms, tags): Self::SystemData) {
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
/// Controlled by the mouse.
/// Goes into an inactive state if the window is not focused (`WindowFocus` resource).
///
/// Can be manually disabled by making the mouse visible using the `HideCursor` resource:
/// `HideCursor.hide = false`
///
/// # Type parameters
///
/// * `T`: This are the keys the `InputHandler` is using for axes and actions. Often, this is a `StringBindings`.
pub struct FreeRotationSystem {
    sensitivity_x: f32,
    sensitivity_y: f32,
    event_reader: Option<ReaderId<Event>>,
}

impl FreeRotationSystem {
    /// Builds a new `FreeRotationSystem` with the specified mouse sensitivity values.
    pub fn new(sensitivity_x: f32, sensitivity_y: f32) -> Self {
        FreeRotationSystem {
            sensitivity_x,
            sensitivity_y,
            event_reader: None,
        }
    }
}

impl<'a> System<'a> for FreeRotationSystem {
    type SystemData = (
        Read<'a, EventChannel<Event>>,
        WriteStorage<'a, TransformComponent>,
        ReadStorage<'a, FlyControlTagComponent>,
        Read<'a, WindowFocus>,
        Read<'a, HideCursor>,
    );

    fn run(&mut self, (events, mut transform, tag, focus, hide): Self::SystemData) {
        let focused = focus.is_focused;
        for event in
            events.read(&mut self.event_reader.as_mut().expect(
                "`FreeRotationSystem::setup` was not called before `FreeRotationSystem::run`",
            ))
        {
            if focused && hide.hide {
                if let Event::DeviceEvent { ref event, .. } = *event {
                    if let DeviceEvent::MouseMotion { delta: (x, y) } = *event {
                        for (transform, _) in (&mut transform, &tag).join() {
                            transform.append_rotation_x_axis(
                                (-y * self.sensitivity_y as f64).to_radians(),
                            );
                            transform.prepend_rotation_y_axis(
                                (-x * self.sensitivity_x as f64).to_radians(),
                            );
                        }
                    }
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::ecs::prelude::SystemData;

        Self::SystemData::setup(res);
        self.event_reader = Some(res.fetch_mut::<EventChannel<Event>>().register_reader());
    }
}

/// A system which reads Events and saves if a window has lost focus in a WindowFocus resource
pub struct MouseFocusUpdateSystem {
    event_reader: Option<ReaderId<Event>>,
}

impl MouseFocusUpdateSystem {
    /// Builds a new MouseFocusUpdateSystem.
    pub fn new() -> MouseFocusUpdateSystem {
        MouseFocusUpdateSystem { event_reader: None }
    }
}

impl<'a> System<'a> for MouseFocusUpdateSystem {
    type SystemData = (Read<'a, EventChannel<Event>>, Write<'a, WindowFocus>);

    fn run(&mut self, (events, mut focus): Self::SystemData) {
        for event in events.read(&mut self.event_reader.as_mut().expect(
            "`MouseFocusUpdateSystem::setup` was not called before `MouseFocusUpdateSystem::run`",
        )) {
            if let Event::WindowEvent { ref event, .. } = *event {
                if let WindowEvent::Focused(focused) = *event {
                    focus.is_focused = focused;
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::ecs::prelude::SystemData;
        Self::SystemData::setup(res);
        self.event_reader = Some(res.fetch_mut::<EventChannel<Event>>().register_reader());
    }
}

/// System which hides the cursor when the window is focused.
/// Requires the usage MouseFocusUpdateSystem at the same time.
pub struct CursorHideSystem {
    is_hidden: bool,
}

impl CursorHideSystem {
    /// Constructs a new CursorHideSystem
    pub fn new() -> CursorHideSystem {
        CursorHideSystem { is_hidden: false }
    }
}

impl<'a> System<'a> for CursorHideSystem {
    type SystemData = (
        ReadExpect<'a, Arc<Window>>,
        Read<'a, HideCursor>,
        Read<'a, WindowFocus>,
    );

    fn run(&mut self, (win, hide, focus): Self::SystemData) {
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

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::ecs::prelude::SystemData;

        Self::SystemData::setup(res);

        let win = res.fetch::<Arc<Window>>();

        if let Err(err) = win.grab_cursor(true) {
            log::error!("Unable to grab the cursor. Error: {:?}", err);
        }
        win.hide_cursor(true);

        self.is_hidden = true;
    }
}
