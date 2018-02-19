use amethyst_core::cgmath::{Deg, Vector3};
use amethyst_core::timing::Time;
use amethyst_core::transform::Transform;
use amethyst_input::InputHandler;
use amethyst_renderer::ScreenDimensions;
use specs::{Component, Fetch, Join, ReadStorage, System, VecStorage, WriteStorage};
use std::hash::Hash;
use std::marker::PhantomData;

/// Add this to a camera if you want it to be a fly camera.
/// You need to add the FlyCameraBundle or the required systems for it to work.
pub struct FlyCameraTag;

impl Component for FlyCameraTag {
    type Storage = VecStorage<FlyCameraTag>;
}

/// The system that manages the camera movement.
/// Generic parameters are the parameters for the InputHandler.
pub struct FlyCameraMovementSystem<A, B> {
    /// The movement speed of the camera in units per second.
    pub speed: f32,
    /// The name of the input axis to locally move in the x coordinates.
    move_x_name: Option<A>,
    /// The name of the input axis to locally move in the y coordinates.
    move_y_name: Option<A>,
    /// The name of the input axis to locally move in the z coordinates.
    move_z_name: Option<A>,
    _marker: PhantomData<B>,
}

impl<A, B> FlyCameraMovementSystem<A, B>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    pub fn new(
        speed: f32,
        move_x_name: Option<A>,
        move_y_name: Option<A>,
        move_z_name: Option<A>,
    ) -> Self {
        FlyCameraMovementSystem {
            speed,
            move_x_name,
            move_y_name,
            move_z_name,
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
        let x = FlyCameraMovementSystem::get_axis(&self.move_x_name, &input);
        let y = FlyCameraMovementSystem::get_axis(&self.move_y_name, &input);
        let z = FlyCameraMovementSystem::get_axis(&self.move_z_name, &input);

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
