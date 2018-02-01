use amethyst_assets::Handle;
use specs::{Entity, WriteStorage};

use resources::{Animation, AnimationCommand, AnimationControl, AnimationSampling, ControlState,
                EndControl};

/// Play a given animation on the given entity.
///
/// ## Parameters:
///
/// - `controls`: animation control storage in the world.
/// - `animation`: handle to the animation to run
/// - `entity`: entity to run the animation on. Must either have an `AnimationHierarchy` that
///             matches the `Animation`, or only refer to a single node, else the animation will
///             not be run.
/// - `end`: action to perform when the animation has reached its end.
pub fn play_animation<T>(
    controls: &mut WriteStorage<AnimationControl<T>>,
    animation: &Handle<Animation<T>>,
    entity: Entity,
    end: EndControl,
) where
    T: AnimationSampling + Send + Sync + 'static,
    T::Channel: Send + Sync + 'static,
    T::Scalar: Send + Sync + 'static,
{
    match controls.get_mut(entity) {
        Some(ref mut control) if control.animation == *animation => {
            control.command = AnimationCommand::Start
        }
        _ => {}
    }
    if let None = controls.get(entity) {
        controls.insert(
            entity,
            AnimationControl::<T>::new(
                animation.clone(),
                end,
                ControlState::Requested,
                AnimationCommand::Start,
            ),
        );
    }
}

/// Pause the running animation on the given entity.
///
/// ## Parameters:
///
/// - `controls`: animation control storage in the world.
/// - `animation`: handle to the animation to run
/// - `entity`: entity the animation is running on. Must either have an `AnimationHierarchy` that
///             matches the `Animation`, or only refer to a single node, else the animation will
///             not be run.
pub fn pause_animation<T>(
    controls: &mut WriteStorage<AnimationControl<T>>,
    animation: &Handle<Animation<T>>,
    entity: Entity,
) where
    T: AnimationSampling + Send + Sync + 'static,
    T::Channel: Send + Sync + 'static,
    T::Scalar: Send + Sync + 'static,
{
    if let Some(ref mut control) = controls.get_mut(entity) {
        if control.animation == *animation && control.state.is_running() {
            control.command = AnimationCommand::Pause;
        }
    }
}

/// Toggle the state between paused and running for the given animation on the given entity.
///
/// ## Parameters:
///
/// - `controls`: animation control storage in the world.
/// - `animation`: handle to the animation
/// - `entity`: entity to run the animation on. Must either have an `AnimationHierarchy` that
///             matches the `Animation`, or only refer to a single node, else the animation will
///             not be run.
/// - `end`: action to perform when the animation has reached its end.
pub fn toggle_animation<T>(
    controls: &mut WriteStorage<AnimationControl<T>>,
    animation: &Handle<Animation<T>>,
    entity: Entity,
    end: EndControl,
) where
    T: AnimationSampling + Send + Sync + 'static,
    T::Channel: Send + Sync + 'static,
    T::Scalar: Send + Sync + 'static,
{
    if controls
        .get(entity)
        .map(|c| c.state.is_running())
        .unwrap_or(false)
    {
        pause_animation(controls, animation, entity);
    } else {
        play_animation(controls, animation, entity, end);
    }
}
