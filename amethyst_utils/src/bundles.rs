use amethyst_core::bundle::{ECSBundle, Result};
use amethyst_renderer::WindowMessages;
use fly_camera::*;
use mouse::*;
use specs::{DispatcherBuilder, World};
use std::hash::Hash;
use std::marker::PhantomData;

/// The bundle that creates a Flying Camera.
/// Note: Will not actually create a camera. It will only register the needed resources and systems.
/// The generic parameters A and B are the ones used in InputHandler<A,B>.
/// You might want to add "cam_move_system" and "cam_rot_system" as dependencies of the TransformSystem.
/// Adding this bundle will grab the mouse, hide it and keep it centered.
pub struct FlyCameraBundle<A, B> {
    sensitivity_x: f32,
    sensitivity_y: f32,
    speed: f32,
    move_x_name: Option<A>,
    move_y_name: Option<A>,
    move_z_name: Option<A>,
    _marker: PhantomData<B>,
}

impl<A, B> FlyCameraBundle<A, B> {
    pub fn new(move_x_name: Option<A>, move_y_name: Option<A>, move_z_name: Option<A>) -> Self {
        FlyCameraBundle {
            sensitivity_x: 1.0,
            sensitivity_y: 1.0,
            speed: 1.0,
            move_x_name,
            move_y_name,
            move_z_name,
            _marker: PhantomData,
        }
    }
    pub fn with_sensitivity(mut self, x: f32, y: f32) -> Self {
        self.sensitivity_x = x;
        self.sensitivity_y = y;
        self
    }
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }
}

impl<'a, 'b, A, B> ECSBundle<'a, 'b> for FlyCameraBundle<A, B>
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    fn build(
        self,
        world: &mut World,
        builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        world.register::<FlyCameraTag>();

        let mut msg = world.res.entry().or_insert_with(|| WindowMessages::new());

        grab_cursor(&mut msg);
        set_mouse_cursor_none(&mut msg);

        Ok(builder
            .add(
                FlyCameraMovementSystem::<A, B>::new(
                    1.0,
                    self.move_x_name,
                    self.move_y_name,
                    self.move_z_name,
                ),
                "cam_move_system",
                &[],
            )
            .add(
                FlyCameraRotationSystem::<A, B>::new(1.0, 1.0),
                "cam_rot_system",
                &[],
            )
            .add(MouseCenterLockSystem, "mouse_lock", &["cam_rot_system"]))
    }
}
