use amethyst_assets::Handle;
use specs::{Entity, WriteStorage};

use resources::{Animation, AnimationCommand, AnimationControl, ControlState, EndControl};

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
pub fn play_animation(
    controls: &mut WriteStorage<AnimationControl>,
    animation: &Handle<Animation>,
    entity: Entity,
    end: EndControl,
) {
    match controls.get_mut(entity) {
        Some(ref mut control) if control.animation == *animation => {
            control.command = AnimationCommand::Start
        }
        _ => {}
    }
    if let None = controls.get(entity) {
        controls.insert(
            entity,
            AnimationControl {
                animation: animation.clone(),
                state: ControlState::Requested,
                command: AnimationCommand::Start,
                end,
            },
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
pub fn pause_animation(
    controls: &mut WriteStorage<AnimationControl>,
    animation: &Handle<Animation>,
    entity: Entity,
) {
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
pub fn toggle_animation(
    controls: &mut WriteStorage<AnimationControl>,
    animation: &Handle<Animation>,
    entity: Entity,
    end: EndControl,
) {
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
