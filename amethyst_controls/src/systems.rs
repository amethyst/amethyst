use amethyst_core::cgmath::{Deg, Vector3};
use amethyst_core::specs::prelude::{Join, Read, ReadExpect, ReadStorage, Resources, System, Write,
                                    WriteStorage};
use amethyst_core::timing::Time;
use amethyst_core::transform::Transform;
use amethyst_input::InputHandler;
use amethyst_renderer::{ScreenDimensions, WindowMessages};
use std::hash::Hash;
use std::marker::PhantomData;

use components::FlyControlTag;

use shrev::{EventChannel, ReaderId};
use winit::{Event, WindowEvent};
use resources::WindowFocus;

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
pub struct FreeRotationSystem<A, B> {
    sensitivity_x: f32,
    sensitivity_y: f32,
    _marker1: PhantomData<A>,
    _marker2: PhantomData<B>,
}

impl<A, B> FreeRotationSystem<A, B> {
    pub fn new(sensitivity_x: f32, sensitivity_y: f32) -> Self {
        FreeRotationSystem {
            sensitivity_x,
            sensitivity_y,
            _marker1: PhantomData,
            _marker2: PhantomData,
        }
    }
}

impl<'a, A, B> System<'a> for FreeRotationSystem<A, B>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    type SystemData = (
        Read<'a, InputHandler<A, B>>,
        Read<'a, WindowFocus>,
        ReadExpect<'a, ScreenDimensions>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, FlyControlTag>,
    );

    fn run(&mut self, (input, focus, dim, mut transform, tag): Self::SystemData) {
        if focus.is_focused {
            // take the same mid-point as the MouseCenterLockSystem
            let half_x = dim.width() as i32 / 2;
            let half_y = dim.height() as i32 / 2;

            if let Some((posx, posy)) = input.mouse_position() {
                let offset_x = half_x as f32 - posx as f32;
                let offset_y = half_y as f32 - posy as f32;
                for (transform, _) in (&mut transform, &tag).join() {
                    transform.pitch_local(Deg(offset_y * self.sensitivity_y));
                    transform.yaw_global(Deg(offset_x * self.sensitivity_x));
                }
            }
        }
    }
}

/// The system that locks the mouse to the center of the screen. Useful for first person camera.
pub struct MouseCenterLockSystem;

impl<'a> System<'a> for MouseCenterLockSystem {
    type SystemData = (
        ReadExpect<'a, ScreenDimensions>,
        Write<'a, WindowMessages>, 
        Write<'a, WindowFocus>
    );

    fn run(&mut self, (dim, mut msg, focus): Self::SystemData) {
        use amethyst_renderer::mouse::*;
        if focus.is_focused {
            let half_x = dim.width() as i32 / 2;
            let half_y = dim.height() as i32 / 2;
            msg.send_command(move |win| {
                if let Err(err) = win.set_cursor_position(half_x, half_y) {
                    error!("Unable to set the cursor position! Error: {:?}", err);
                }

            });
        } else {
            release_cursor(&mut msg);
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::specs::prelude::SystemData;
        use amethyst_renderer::mouse::*;
        Self::SystemData::setup(res);
        let mut msg = res.fetch_mut::<WindowMessages>();
        grab_cursor(&mut msg);
        set_mouse_cursor_none(&mut msg);
    }
}

// A system which reads Events and saves if a window has lost focus in a WindowFocus resource
pub struct MouseFocusUpdateSystem {
    event_reader: Option<ReaderId<Event>>,
}

impl MouseFocusUpdateSystem {
    pub fn new() -> MouseFocusUpdateSystem {
        MouseFocusUpdateSystem { 
            event_reader: None 
        }
    }
}

impl<'a> System<'a> for MouseFocusUpdateSystem {
    type SystemData = (
        Read<'a, EventChannel<Event>>,
        Write<'a, WindowFocus>,
    );

    fn run(&mut self, (events, mut focus): Self::SystemData) {
        for event in events.read(&mut self.event_reader.as_mut().unwrap()) {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Focused(focused) => {
                        focus.is_focused = *focused;
                    },
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