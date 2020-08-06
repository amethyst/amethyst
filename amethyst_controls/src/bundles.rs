use std::marker::PhantomData;

use amethyst_core::{ecs::*, math::one, shrev::EventChannel};
use amethyst_error::Error;
use amethyst_input::BindingTypes;

use winit::Event;

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
    fn load(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        builder.add_system(build_fly_movement_system::<T>(
            self.speed,
            self.horizontal_axis.clone(),
            self.vertical_axis.clone(),
            self.longitudinal_axis.clone(),
        ));

        let reader = resources
            .get_mut::<EventChannel<Event>>()
            .expect("Window event channel not found in resources")
            .register_reader();

        builder.add_system(build_free_rotation_system(
            self.sensitivity_x,
            self.sensitivity_y,
            reader,
        ));

        resources.insert(WindowFocus::new());

        let reader = resources
            .get_mut::<EventChannel<Event>>()
            .expect("Window event channel not found in resources")
            .register_reader();

        builder.add_system(build_mouse_focus_update_system(reader));

        resources.insert(HideCursor::default());
        builder.add_system(build_cursor_hide_system());

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
    fn load(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        let reader = resources
            .get_mut::<EventChannel<Event>>()
            .expect("Window event channel not found in resources")
            .register_reader();

        builder.add_system(build_free_rotation_system(
            self.sensitivity_x,
            self.sensitivity_y,
            reader,
        ));
        builder.add_system(build_arc_ball_rotation_system());

        resources.insert(WindowFocus::new());

        let reader = resources
            .get_mut::<EventChannel<Event>>()
            .expect("Window event channel not found in resources")
            .register_reader();

        builder.add_system(build_mouse_focus_update_system(reader));

        resources.insert(HideCursor::default());
        builder.add_system(build_cursor_hide_system());

        Ok(())
    }
}
