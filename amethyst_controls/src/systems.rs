use std::marker::PhantomData;
use std::hash::Hash;

use amethyst_core::cgmath::{Deg, Vector3};
use amethyst_core::specs::prelude::{Join, Read, ReadStorage, Resources, System, Write,
                                    WriteStorage};
use amethyst_core::timing::Time;
use amethyst_core::transform::Transform;
use amethyst_input::InputHandler;
use amethyst_renderer::WindowMessages;
use amethyst_core::shrev::{EventChannel, ReaderId};
use winit::{Event, WindowEvent, DeviceEvent};

use components::FlyControlTag;

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

/// The system that manages the view rotation.
/// Controlled by the mouse.
/// Handles raw mouse input.
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
    );

    fn run(&mut self, (events, mut transform, tag): Self::SystemData) {
        for event in events.read(&mut self.event_reader.as_mut().unwrap()) {
            match event {
                &Event::DeviceEvent { ref event, .. } => {
                    match event {
                        &DeviceEvent::MouseMotion  { delta: (x, y) } => {
                            for (transform, _) in (&mut transform, &tag).join() {
                                transform.pitch_local(Deg( (-1.0) * y as f32 * self.sensitivity_y));
                                transform.yaw_global(Deg(  (-1.0) * x as f32 * self.sensitivity_x));
                            }
                        },
                        _ => (),
                    }
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

/// A system which hides the cursor and centers it so it can't leave the window
/// Releases cursor if window focus is lost
pub struct CursorHideSystem {

    event_reader: Option<ReaderId<Event>>,
}

impl CursorHideSystem {
    pub fn new() -> CursorHideSystem {
        CursorHideSystem {
            event_reader: None
        }
    }
}

impl<'a> System<'a> for CursorHideSystem {
    type SystemData = (
        Read<'a, EventChannel<Event>>,
        Write<'a, WindowMessages>,
    );

    fn run(&mut self, (events, mut msg): Self::SystemData) {
        use amethyst_renderer::mouse::*;

        for event in events.read(&mut self.event_reader.as_mut().unwrap()) {
            match event {
                &Event::WindowEvent { ref event, .. } => match event {
                    &WindowEvent::Focused(focused) => {
                        if focused { grab_cursor(&mut msg) } else { release_cursor(&mut msg) }
                    },
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
