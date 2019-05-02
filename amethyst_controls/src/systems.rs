use std::{hash::Hash, marker::PhantomData};

use winit::{DeviceEvent, Event, WindowEvent};

use crate::{
    components::{ArcBallControlTag, FlyControlTag},
    resources::{HideCursor, WindowFocus},
};
use amethyst_core::{
    ecs::prelude::{Join, Read, ReadStorage, Resources, System, Write, WriteStorage},
    math::{convert, RealField, Unit, Vector3},
    shrev::{EventChannel, ReaderId},
    timing::Time,
    transform::Transform,
};
use amethyst_input::{get_input_axis_simple, InputHandler};
use amethyst_renderer::WindowMessages;

/// The system that manages the fly movement.
///
/// # Type parameters
///
/// * `A`: This is the key the `InputHandler` is using for axes. Often, this is a `String`.
/// * `B`: This is the key the `InputHandler` is using for actions. Often, this is a `String`.
/// * `N`: RealField bound (f32 or f64).
pub struct FlyMovementSystem<A, B, N> {
    /// The movement speed of the movement in units per second.
    speed: N,
    /// The name of the input axis to locally move in the x coordinates.
    right_input_axis: Option<A>,
    /// The name of the input axis to locally move in the y coordinates.
    up_input_axis: Option<A>,
    /// The name of the input axis to locally move in the z coordinates.
    forward_input_axis: Option<A>,
    _marker: PhantomData<(B, N)>,
}

impl<A, B, N> FlyMovementSystem<A, B, N>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    /// Builds a new `FlyMovementSystem` using the provided speeds and axis controls.
    pub fn new(
        speed: N,
        right_input_axis: Option<A>,
        up_input_axis: Option<A>,
        forward_input_axis: Option<A>,
    ) -> Self {
        FlyMovementSystem {
            speed,
            right_input_axis,
            up_input_axis,
            forward_input_axis,
            _marker: PhantomData,
        }
    }
}

impl<'a, A, B, N> System<'a> for FlyMovementSystem<A, B, N>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
    N: RealField,
{
    type SystemData = (
        Read<'a, Time>,
        WriteStorage<'a, Transform<N>>,
        Read<'a, InputHandler<A, B>>,
        ReadStorage<'a, FlyControlTag>,
    );

    fn run(&mut self, (time, mut transform, input, tag): Self::SystemData) {
        let x = get_input_axis_simple(&self.right_input_axis, &input);
        let y = get_input_axis_simple(&self.up_input_axis, &input);
        let z = get_input_axis_simple(&self.forward_input_axis, &input);

        if let Some(dir) = Unit::try_new(Vector3::new(x, y, z), convert(1.0e-6)) {
            for (transform, _) in (&mut transform, &tag).join() {
                let delta_sec: N = convert(time.delta_seconds() as f64);
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
///
/// # Type parameters
///
/// * `N`: RealField bound (f32 or f64).
pub struct ArcBallRotationSystem<N>(PhantomData<N>);

impl<N> Default for ArcBallRotationSystem<N> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<'a, N: RealField> System<'a> for ArcBallRotationSystem<N> {
    type SystemData = (
        WriteStorage<'a, Transform<N>>,
        ReadStorage<'a, ArcBallControlTag<N>>,
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
/// * `A`: This is the key the `InputHandler` is using for axes. Often, this is a `String`.
/// * `B`: This is the key the `InputHandler` is using for actions. Often, this is a `String`.
/// * `N`: RealField bound (f32 or f64).
pub struct FreeRotationSystem<A, B, N: RealField> {
    sensitivity_x: f32,
    sensitivity_y: f32,
    _marker1: PhantomData<A>,
    _marker2: PhantomData<B>,
    _marker3: PhantomData<N>,
    event_reader: Option<ReaderId<Event>>,
}

impl<A, B, N: RealField> FreeRotationSystem<A, B, N> {
    /// Builds a new `FreeRotationSystem` with the specified mouse sensitivity values.
    pub fn new(sensitivity_x: f32, sensitivity_y: f32) -> Self {
        FreeRotationSystem {
            sensitivity_x,
            sensitivity_y,
            _marker1: PhantomData,
            _marker2: PhantomData,
            _marker3: PhantomData,
            event_reader: None,
        }
    }
}

impl<'a, A, B, N> System<'a> for FreeRotationSystem<A, B, N>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
    N: RealField,
{
    type SystemData = (
        Read<'a, EventChannel<Event>>,
        WriteStorage<'a, Transform<N>>,
        ReadStorage<'a, FlyControlTag>,
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
                            transform.append_rotation_x_axis(convert(
                                (-y * self.sensitivity_y as f64).to_radians(),
                            ));
                            transform.prepend_rotation_y_axis(convert(
                                (-x * self.sensitivity_x as f64).to_radians(),
                            ));
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
        Write<'a, WindowMessages>,
        Read<'a, HideCursor>,
        Read<'a, WindowFocus>,
    );

    fn run(&mut self, (mut msg, hide, focus): Self::SystemData) {
        use amethyst_renderer::mouse::*;
        if focus.is_focused {
            if !self.is_hidden && hide.hide {
                grab_cursor(&mut msg);
                hide_cursor(&mut msg);
                self.is_hidden = true;
            } else if self.is_hidden && !hide.hide {
                release_cursor(&mut msg);
                self.is_hidden = false;
            }
        } else if self.is_hidden {
            release_cursor(&mut msg);
            self.is_hidden = false;
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::ecs::prelude::SystemData;
        use amethyst_renderer::mouse::*;

        Self::SystemData::setup(res);

        let mut msg = res.fetch_mut::<WindowMessages>();
        grab_cursor(&mut msg);
        hide_cursor(&mut msg);
        self.is_hidden = true;
    }
}
