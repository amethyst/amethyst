use std::marker::PhantomData;

use amethyst_core::{bundle::SystemBundle, ecs::prelude::DispatcherBuilder, math::one};
use amethyst_error::Error;
use amethyst_input::BindingTypes;

use super::*;

use derive_new::new;

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
#[derive(new, Debug)]
pub struct FlyControlBundle<T: BindingTypes> {
    #[new(value = "1.0")]
    sensitivity_x: f32,
    #[new(value = "1.0")]
    sensitivity_y: f32,
    #[new(value = "one()")]
    speed: f32,
    right_input_axis: Option<T::Axis>,
    up_input_axis: Option<T::Axis>,
    forward_input_axis: Option<T::Axis>,
}

impl<T: BindingTypes> FlyControlBundle<T> {
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
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(
            FlyMovementSystem::<T>::new(
                self.speed,
                self.right_input_axis,
                self.up_input_axis,
                self.forward_input_axis,
            ),
            "fly_movement",
            &[],
        );
        builder.add(
            FreeRotationSystem::new(self.sensitivity_x, self.sensitivity_y),
            "free_rotation",
            &[],
        );
        builder.add(
            MouseFocusUpdateSystem::new(),
            "mouse_focus",
            &["free_rotation"],
        );
        builder.add(CursorHideSystem::new(), "cursor_hide", &["mouse_focus"]);
        Ok(())
    }
}

/// The bundle that creates an arc ball movement system.
/// Note: Will not actually create a moving entity. It will only register the needed resources and systems.
/// The generic parameters A and B are the ones used in InputHandler<A,B>.
/// You might want to add "fly_movement" and "free_rotation" as dependencies of the TransformSystem.
/// Adding this bundle will grab the mouse, hide it and keep it centered.
/// See the `arc_ball_camera` example to see how to use the arc ball camera.
#[derive(new, Debug)]
pub struct ArcBallControlBundle<T: BindingTypes> {
    #[new(value = "1.0")]
    sensitivity_x: f32,
    #[new(value = "1.0")]
    sensitivity_y: f32,
    _marker: PhantomData<T>,
}

impl<T: BindingTypes> ArcBallControlBundle<T> {
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
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(ArcBallRotationSystem::default(), "arc_ball_rotation", &[]);
        builder.add(
            FreeRotationSystem::new(self.sensitivity_x, self.sensitivity_y),
            "free_rotation",
            &[],
        );
        builder.add(
            MouseFocusUpdateSystem::new(),
            "mouse_focus",
            &["free_rotation"],
        );
        builder.add(CursorHideSystem::new(), "cursor_hide", &["mouse_focus"]);
        Ok(())
    }
}
