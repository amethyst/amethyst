use std::marker::PhantomData;

use amethyst_core::{
    bundle::SystemBundle,
    ecs::prelude::{DispatcherBuilder, World},
    math::one,
    SystemDesc,
};
use amethyst_error::Error;
use amethyst_input::BindingTypes;

use super::*;

/// The bundle that creates a flying movement system.
///
/// Note: Will not actually create a moving entity. It will only register the needed resources and
/// systems.
///
/// You might want to add `"fly_movement"` and `"free_rotation"` as dependencies of the
/// `TransformSystem` in order to apply changes made by these systems in the same frame.
/// Adding this bundle will grab the mouse, hide it and keep it centered.
///
/// # Type parameters
///
/// * `T`: This are the keys the `InputHandler` is using for axes and actions. Often, this is a `StringBindings`.
///
/// # Systems
///
/// This bundle adds the following systems:
///
/// * `FlyMovementSystem`
/// * `FreeRotationSystem`
/// * `MouseFocusUpdateSystem`
/// * `CursorHideSystem`
#[derive(Debug)]
pub struct FlyControlBundle<T: BindingTypes> {
    sensitivity_x: f32,
    sensitivity_y: f32,
    speed: f32,
    right_input_axis: Option<T::Axis>,
    up_input_axis: Option<T::Axis>,
    forward_input_axis: Option<T::Axis>,
}

impl<T: BindingTypes> FlyControlBundle<T> {
    /// Builds a new fly control bundle using the provided axes as controls.
    pub fn new(
        right_input_axis: Option<T::Axis>,
        up_input_axis: Option<T::Axis>,
        forward_input_axis: Option<T::Axis>,
    ) -> Self {
        FlyControlBundle {
            sensitivity_x: 1.0,
            sensitivity_y: 1.0,
            speed: one(),
            right_input_axis,
            up_input_axis,
            forward_input_axis,
        }
    }

    /// Alters the mouse sensitivy on this `FlyControlBundle`
    pub fn with_sensitivity(mut self, x: f32, y: f32) -> Self {
        self.sensitivity_x = x;
        self.sensitivity_y = y;
        self
    }

    /// Alters the speed on this `FlyControlBundle`.
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }
}

impl<'a, 'b, T: BindingTypes> SystemBundle<'a, 'b> for FlyControlBundle<T> {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add(
            FlyMovementSystemDesc::<T>::new(
                self.speed,
                self.right_input_axis,
                self.up_input_axis,
                self.forward_input_axis,
            )
            .build(world),
            "fly_movement",
            &[],
        );
        builder.add(
            FreeRotationSystemDesc::new(self.sensitivity_x, self.sensitivity_y).build(world),
            "free_rotation",
            &[],
        );
        builder.add_thread_local(
            MouseFocusUpdateSystemDesc::default().build(world)
        );
        builder.add_thread_local(
            CursorHideSystemDesc::default().build(world)
        );
        Ok(())
    }
}

/// The bundle that creates an arc ball movement system.
/// Note: Will not actually create a moving entity. It will only register the needed resources and systems.
/// The generic parameters A and B are the ones used in InputHandler<A,B>.
/// You might want to add "fly_movement" and "free_rotation" as dependencies of the TransformSystem.
/// Adding this bundle will grab the mouse, hide it and keep it centered.
///
/// See the `arc_ball_camera` example to see how to use the arc ball camera.
#[derive(Debug)]
pub struct ArcBallControlBundle<T: BindingTypes> {
    sensitivity_x: f32,
    sensitivity_y: f32,
    _marker: PhantomData<T>,
}

impl<T: BindingTypes> ArcBallControlBundle<T> {
    /// Builds a new `ArcBallControlBundle` with a default sensitivity of 1.0
    pub fn new() -> Self {
        ArcBallControlBundle {
            sensitivity_x: 1.0,
            sensitivity_y: 1.0,
            _marker: PhantomData,
        }
    }

    /// Builds a new `ArcBallControlBundle` with the provided mouse sensitivity values.
    pub fn with_sensitivity(mut self, x: f32, y: f32) -> Self {
        self.sensitivity_x = x;
        self.sensitivity_y = y;
        self
    }
}

impl<T: BindingTypes> Default for ArcBallControlBundle<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, 'b, T: BindingTypes> SystemBundle<'a, 'b> for ArcBallControlBundle<T> {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add(ArcBallRotationSystem::default(), "arc_ball_rotation", &[]);
        builder.add(
            FreeRotationSystemDesc::new(self.sensitivity_x, self.sensitivity_y).build(world),
            "free_rotation",
            &[],
        );
        builder.add(
            MouseFocusUpdateSystemDesc::default().build(world),
            "mouse_focus",
            &["free_rotation"],
        );
        builder.add(
            CursorHideSystemDesc::default().build(world),
            "cursor_hide",
            &["mouse_focus"],
        );
        Ok(())
    }
}
