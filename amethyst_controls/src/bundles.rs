use std::marker::PhantomData;

use amethyst_core::{
    dispatcher::{DispatcherBuilder, Stage, SystemBundle},
    ecs::prelude::*,
    math::one,
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
    horizontal_axis: Option<T::Axis>,
    vertical_axis: Option<T::Axis>,
    longitudinal_axis: Option<T::Axis>,
}

impl<T: BindingTypes> FlyControlBundle<T> {
    /// Builds a new fly control bundle using the provided axes as controls.
    pub fn new(
        horizontal_axis: Option<T::Axis>,
        vertical_axis: Option<T::Axis>,
        longitudinal_axis: Option<T::Axis>,
    ) -> Self {
        FlyControlBundle {
            sensitivity_x: 1.0,
            sensitivity_y: 1.0,
            speed: one(),
            horizontal_axis,
            vertical_axis,
            longitudinal_axis,
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

impl<T: BindingTypes> SystemBundle for FlyControlBundle<T> {
    fn build(
        self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder<'_>,
    ) -> Result<(), Error> {
        builder.add_system(
            Stage::Begin,
            build_fly_movement_system::<T>(
                self.speed,
                self.horizontal_axis,
                self.vertical_axis,
                self.longitudinal_axis,
            ),
        );
        builder.add_system(
            Stage::Begin,
            build_free_rotation_system(self.sensitivity_x, self.sensitivity_y),
        );
        builder.add_system(Stage::Begin, build_mouse_focus_update_system);
        builder.add_system(Stage::Begin, build_cursor_hide_system);
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

impl<T: BindingTypes> SystemBundle for ArcBallControlBundle<T> {
    fn build(
        self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder<'_>,
    ) -> Result<(), Error> {
        builder.add_system(
            Stage::Begin,
            build_free_rotation_system(self.sensitivity_x, self.sensitivity_y),
        );
        builder.add_system(Stage::Begin, build_arc_ball_rotation_system);
        builder.add_system(Stage::Begin, build_mouse_focus_update_system);
        builder.add_system(Stage::Begin, build_cursor_hide_system);
        Ok(())
    }
}
