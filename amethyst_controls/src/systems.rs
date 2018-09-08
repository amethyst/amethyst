use amethyst_core::cgmath::{Deg, Vector3};
use amethyst_core::shrev::{EventChannel, ReaderId};
use amethyst_core::specs::prelude::{
    Join, Read, ReadStorage, Resources, System, Write, WriteStorage,
};
use amethyst_core::timing::Time;
use amethyst_core::transform::Transform;
use amethyst_input::InputHandler;
use amethyst_renderer::WindowMessages;
use components::{ArcBallControlTag, FlyControlTag, XYControlTag};
use resources::WindowFocus;
use std::hash::Hash;
use std::marker::PhantomData;
use winit::{DeviceEvent, Event, WindowEvent};

/// The system that manages the fly movement.
/// Generic parameters are the parameters for the InputHandler.
pub struct FlyMovementSystem<A, B> {
    /// The movement speed of the movement in units per second.
    speed: f32,
    /// The name of the input axis to locally move in the x coordinates.
    right_input_axis: Option<A>,
    /// The name of the input axis to locally move in the y coordinates.
    up_input_axis: Option<A>,
    /// The name of the input axis to locally move in the z coordinates.
    forward_input_axis: Option<A>,
    _marker: PhantomData<B>,
}

impl<A, B> FlyMovementSystem<A, B>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    pub fn new(
        speed: f32,
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

    fn get_axis(name: &Option<A>, input: &InputHandler<A, B>) -> f32 {
        name.as_ref()
            .and_then(|ref n| input.axis_value(n))
            .unwrap_or(0.0) as f32
    }
}

impl<'a, A, B> System<'a> for FlyMovementSystem<A, B>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    type SystemData = (
        Read<'a, Time>,
        WriteStorage<'a, Transform>,
        Read<'a, InputHandler<A, B>>,
        ReadStorage<'a, FlyControlTag>,
    );

    fn run(&mut self, (time, mut transform, input, tag): Self::SystemData) {
        let x = FlyMovementSystem::get_axis(&self.right_input_axis, &input);
        let y = FlyMovementSystem::get_axis(&self.up_input_axis, &input);
        let z = FlyMovementSystem::get_axis(&self.forward_input_axis, &input);

        let dir = Vector3::new(x, y, z);

        for (transform, _) in (&mut transform, &tag).join() {
            transform.move_along_local(dir, time.delta_seconds() * self.speed);
        }
    }
}

/// System moving any entity holding a `XYControlTag` component along the
/// X and Y axis. It can also zoom using the transform's scale property.
#[derive(Default)]
pub struct XYCameraSystem<A, B> {
    /// The name of the input axis to locally move in the x coordinates.
    pub x_axis: Option<A>,
    /// The name of the input axis to locally move in the y coordinates.
    pub y_axis: Option<A>,
    /// The name of the input axis to zoom.
    pub zoom_axis: Option<A>,
    _marker: PhantomData<B>,
}

impl<A, B> XYCameraSystem<A, B>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    /// Creates the camera system.
    pub fn new(x_axis: impl Into<A>, y_axis: impl Into<A>) -> Self {
        Self {
            x_axis: Some(x_axis.into()),
            y_axis: Some(y_axis.into()),
            zoom_axis: None,
            _marker: PhantomData,
        }
    }

    /// Allows to control the zoom level.
    pub fn with_zoom(mut self, zoom_axis: impl Into<A>) -> Self {
        self.zoom_axis = Some(zoom_axis.into());
        self
    }
}

impl<'a, A, B> System<'a> for XYCameraSystem<A, B>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    type SystemData = (
        ReadStorage<'a, XYControlTag>,
        Read<'a, Time>,
        Read<'a, InputHandler<A, B>>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (cameras, time, input, mut transf): Self::SystemData) {
        for (camera, mut camera_transf) in (&cameras, &mut transf).join() {
            if let Some(ref x_axis) = self.x_axis {
                if let Some(horiz) = input.axis_value(x_axis) {
                    camera_transf.translation.x = (camera_transf.translation.x
                        + camera.x_speed * time.delta_seconds() * horiz as f32)
                        .max(camera.horizontal_borders.0)
                        .min(camera.horizontal_borders.1);
                }
            }
            if let Some(ref y_axis) = self.y_axis {
                if let Some(vert) = input.axis_value(y_axis) {
                    camera_transf.translation.y = (camera_transf.translation.y
                        + camera.y_speed * time.delta_seconds() * vert as f32)
                        .max(camera.vertical_borders.0)
                        .min(camera.vertical_borders.1);
                }
            }
            if let Some(ref zoom_axis) = self.zoom_axis {
                if let Some(zoom) = input.axis_value(zoom_axis) {
                    camera_transf.scale.x = (camera_transf.scale.x
                        + camera.zoom_speed * zoom as f32)
                        .max(camera.zoom_borders.0)
                        .min(camera.zoom_borders.1);
                    camera_transf.scale.y = (camera_transf.scale.y
                        + camera.zoom_speed * zoom as f32)
                        .max(camera.zoom_borders.0)
                        .min(camera.zoom_borders.1);
                }
            }
        }
    }
}

/// The system that manages the arc ball movement;
/// In essence, the system will allign the camera with its target while keeping the distance to it
/// and while keeping the orientation of the camera.
/// To modify the orientation of the camera in accordance with the mouse input, please use the
/// FreeRotationSystem.
#[derive(Default)]
pub struct ArcBallMovementSystem;

impl<'a> System<'a> for ArcBallMovementSystem {
    type SystemData = (
        WriteStorage<'a, Transform>,
        ReadStorage<'a, ArcBallControlTag>,
    );

    fn run(&mut self, (mut transforms, tags): Self::SystemData) {
        let mut position = None;
        for (transform, arc_ball_camera_tag) in (&transforms, &tags).join() {
            let pos_vec = transform.rotation * -Vector3::unit_z() * arc_ball_camera_tag.distance;
            if let Some(target_transform) = transforms.get(arc_ball_camera_tag.target) {
                position = Some(target_transform.translation - pos_vec);
            }
        }
        if let Some(new_pos) = position {
            for (transform, _) in (&mut transforms, &tags).join() {
                transform.translation = new_pos;
            }
        }
    }
}

/// The system that manages the view rotation.
/// Controlled by the mouse.
pub struct FreeRotationSystem<A, B> {
    sensitivity_x: f32,
    sensitivity_y: f32,
    _marker1: PhantomData<A>,
    _marker2: PhantomData<B>,
    event_reader: Option<ReaderId<Event>>,
}

impl<A, B> FreeRotationSystem<A, B> {
    pub fn new(sensitivity_x: f32, sensitivity_y: f32) -> Self {
        FreeRotationSystem {
            sensitivity_x,
            sensitivity_y,
            _marker1: PhantomData,
            _marker2: PhantomData,
            event_reader: None,
        }
    }
}

impl<'a, A, B> System<'a> for FreeRotationSystem<A, B>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    type SystemData = (
        Read<'a, EventChannel<Event>>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, FlyControlTag>,
        Read<'a, WindowFocus>,
    );

    fn run(&mut self, (events, mut transform, tag, focus): Self::SystemData) {
        let focused = focus.is_focused;
        for event in events.read(&mut self.event_reader.as_mut().unwrap()) {
            if focused {
                match *event {
                    Event::DeviceEvent { ref event, .. } => match *event {
                        DeviceEvent::MouseMotion { delta: (x, y) } => {
                            for (transform, _) in (&mut transform, &tag).join() {
                                transform.pitch_local(Deg((-1.0) * y as f32 * self.sensitivity_y));
                                transform.yaw_global(Deg((-1.0) * x as f32 * self.sensitivity_x));
                            }
                        }
                        _ => (),
                    },
                    _ => (),
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::specs::prelude::SystemData;

        Self::SystemData::setup(res);
        self.event_reader = Some(res.fetch_mut::<EventChannel<Event>>().register_reader());
    }
}

/// A system which reads Events and saves if a window has lost focus in a WindowFocus resource
pub struct MouseFocusUpdateSystem {
    event_reader: Option<ReaderId<Event>>,
}

impl MouseFocusUpdateSystem {
    pub fn new() -> MouseFocusUpdateSystem {
        MouseFocusUpdateSystem { event_reader: None }
    }
}

impl<'a> System<'a> for MouseFocusUpdateSystem {
    type SystemData = (Read<'a, EventChannel<Event>>, Write<'a, WindowFocus>);

    fn run(&mut self, (events, mut focus): Self::SystemData) {
        for event in events.read(&mut self.event_reader.as_mut().unwrap()) {
            match event {
                &Event::WindowEvent { ref event, .. } => match event {
                    &WindowEvent::Focused(focused) => {
                        focus.is_focused = focused;
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::specs::prelude::SystemData;
        Self::SystemData::setup(res);
        self.event_reader = Some(res.fetch_mut::<EventChannel<Event>>().register_reader());
    }
}

// System which hides the cursor when the window is focused
pub struct CursorHideSystem {
    event_reader: Option<ReaderId<Event>>,
}

impl CursorHideSystem {
    pub fn new() -> CursorHideSystem {
        CursorHideSystem { event_reader: None }
    }
}

impl<'a> System<'a> for CursorHideSystem {
    type SystemData = (Read<'a, EventChannel<Event>>, Write<'a, WindowMessages>);

    fn run(&mut self, (events, mut msg): Self::SystemData) {
        use amethyst_renderer::mouse::*;

        for event in events.read(&mut self.event_reader.as_mut().unwrap()) {
            match *event {
                Event::WindowEvent { ref event, .. } => match event {
                    &WindowEvent::Focused(focused) => {
                        if focused {
                            grab_cursor(&mut msg)
                        } else {
                            release_cursor(&mut msg)
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::specs::prelude::SystemData;
        use amethyst_renderer::mouse::*;

        Self::SystemData::setup(res);
        self.event_reader = Some(res.fetch_mut::<EventChannel<Event>>().register_reader());

        let mut msg = res.fetch_mut::<WindowMessages>();
        grab_cursor(&mut msg);
        set_mouse_cursor_none(&mut msg);
    }
}
