use amethyst_core::cgmath::{Deg, Vector3};
use amethyst_core::timing::Time;
use amethyst_core::transform::Transform;
use amethyst_input::InputHandler;
use amethyst_renderer::ScreenDimensions;
use specs::{Component, Fetch, Join, NullStorage, ReadStorage, System, WriteStorage};
use std::hash::Hash;
use std::marker::PhantomData;

/// Add this to a camera if you want it to be a fly camera.
/// You need to add the FlyCameraBundle or the required systems for it to work.
#[derive(Default)]
pub struct FlyCameraTag;

impl Component for FlyCameraTag {
    type Storage = NullStorage<FlyCameraTag>;
}

/// The system that manages the camera movement.
/// Generic parameters are the parameters for the InputHandler.
pub struct FlyCameraMovementSystem<A, B> {
    /// The movement speed of the camera in units per second.
    speed: f32,
    /// The name of the input axis to locally move in the x coordinates.
    right_input_axis: Option<A>,
    /// The name of the input axis to locally move in the y coordinates.
    up_input_axis: Option<A>,
    /// The name of the input axis to locally move in the z coordinates.
    forward_input_axis: Option<A>,
    _marker: PhantomData<B>,
}

impl<A, B> FlyCameraMovementSystem<A, B>
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
        FlyCameraMovementSystem {
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

impl<'a, A, B> System<'a> for FlyCameraMovementSystem<A, B>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    type SystemData = (
        Fetch<'a, Time>,
        WriteStorage<'a, Transform>,
        Fetch<'a, InputHandler<A, B>>,
        ReadStorage<'a, FlyCameraTag>,
    );

    fn run(&mut self, (time, mut transform, input, tag): Self::SystemData) {
        let x = FlyCameraMovementSystem::get_axis(&self.right_input_axis, &input);
        let y = FlyCameraMovementSystem::get_axis(&self.up_input_axis, &input);
        let z = FlyCameraMovementSystem::get_axis(&self.forward_input_axis, &input);

        let dir = Vector3::new(x, y, z);

        for (transform, _) in (&mut transform, &tag).join() {
            transform.move_local(dir, time.delta_seconds() * self.speed);
        }
    }
}

/// The system that manages the camera rotation.
pub struct FlyCameraRotationSystem<A, B> {
    sensitivity_x: f32,
    sensitivity_y: f32,
    _marker1: PhantomData<A>,
    _marker2: PhantomData<B>,
}

impl<A, B> FlyCameraRotationSystem<A, B> {
    pub fn new(sensitivity_x: f32, sensitivity_y: f32) -> Self {
        FlyCameraRotationSystem {
            sensitivity_x,
            sensitivity_y,
            _marker1: PhantomData,
            _marker2: PhantomData,
        }
    }
}

impl<'a, A, B> System<'a> for FlyCameraRotationSystem<A, B>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    type SystemData = (
        Fetch<'a, InputHandler<A, B>>,
        Fetch<'a, ScreenDimensions>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, FlyCameraTag>,
    );

    fn run(&mut self, (input, dim, mut transform, tag): Self::SystemData) {
        let half_x = dim.width() / 2.0;
        let half_y = dim.height() / 2.0;
        if let Some((posx, posy)) = input.mouse_position() {
            let offset_x = half_x - posx as f32;
            let offset_y = half_y - posy as f32;
            for (transform, _) in (&mut transform, &tag).join() {
                transform.rotate_local(
                    Vector3::new(1.0, 0.0, 0.0),
                    Deg(offset_y * self.sensitivity_y),
                );
                transform.rotate_global(
                    Vector3::new(0.0, 1.0, 0.0),
                    Deg(offset_x * self.sensitivity_x),
                );
            }
        }
    }
}
