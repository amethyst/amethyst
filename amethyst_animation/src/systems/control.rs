use std::marker;
use std::time::Duration;

use amethyst_assets::{AssetStorage, Handle};
use minterpolate::InterpolationPrimitive;
use specs::{Component, Entities, Entity, Fetch, Join, ReadStorage, System, WriteStorage};

use resources::{Animation, AnimationCommand, AnimationControl, AnimationControlSet,
                AnimationHierarchy, AnimationSampling, ControlState, Sampler, SamplerControl,
                SamplerControlSet, StepDirection};

/// System for setting up animations, should run before `SamplerInterpolationSystem`.
///
/// Will process all active `AnimationControl` + `AnimationHierarchy`, and do processing of the
/// animations they describe. If an animation only targets a single node/entity, there is no need
/// for `AnimationHierarchy`.
#[derive(Default)]
pub struct AnimationControlSystem<I, T> {
    m: marker::PhantomData<(I, T)>,
    next_id: u64,
}

impl<I, T> AnimationControlSystem<I, T> {
    pub fn new() -> Self {
        Self {
            m: marker::PhantomData,
            next_id: 1,
        }
    }
}

impl<'a, I, T> System<'a> for AnimationControlSystem<I, T>
where
    I: PartialEq + Copy + Send + Sync + 'static,
    T: AnimationSampling + Component,
{
    type SystemData = (
        Entities<'a>,
        Fetch<'a, AssetStorage<Animation<T>>>,
        Fetch<'a, AssetStorage<Sampler<T::Primitive>>>,
        WriteStorage<'a, AnimationControlSet<I, T>>,
        WriteStorage<'a, SamplerControlSet<T>>,
        ReadStorage<'a, AnimationHierarchy<T>>,
        ReadStorage<'a, T>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            animation_storage,
            sampler_storage,
            mut controls,
            mut samplers,
            hierarchies,
            transforms,
        ) = data;
        let mut remove_sets = Vec::default();
        for (entity, control_set) in (&*entities, &mut controls).join() {
            let mut remove_ids = Vec::default();
            for &mut (ref id, ref mut control) in control_set.animations.iter_mut() {
                let mut remove = false;
                if let Some(state) = animation_storage.get(&control.animation).and_then(
                    |animation| {
                        process_animation_control(
                            &entity,
                            animation,
                            control,
                            hierarchies.get(entity),
                            &*sampler_storage,
                            &mut samplers,
                            &transforms,
                            &mut remove,
                            &mut self.next_id,
                        )
                    },
                ) {
                    control.state = state;
                }
                if let AnimationCommand::Step(_) = control.command {
                    control.command = AnimationCommand::Start;
                }
                if let AnimationCommand::SetInputValue(_) = control.command {
                    control.command = AnimationCommand::Start;
                }
                if remove {
                    remove_ids.push(*id);
                }
            }
            for id in remove_ids {
                control_set.remove(id);
                if control_set.is_empty() {
                    remove_sets.push(entity);
                }
            }
        }

        for entity in remove_sets {
            controls.remove(entity);
        }
    }
}

/// Check if the given animation list is for a single node. If so, we don't need an
/// `AnimationHierarchy`.
fn only_one_index<C, P>(nodes: &[(usize, C, Handle<Sampler<P>>)]) -> bool
where
    P: InterpolationPrimitive,
{
    if nodes.is_empty() {
        true
    } else {
        let first = nodes[0].0;
        nodes.iter().all(|&(ref i, _, _)| *i == first)
    }
}

/// Process a single animation control object.
///
/// ## Parameters:
///
/// - `entity`: the entity the control object is active for
/// - `animation`: the animation the control is for
/// - `control`: animation control object
/// - `hierarchy`: the animation node hierarchy for the entity hierarchy the animation instance is
///                active for, if this is None the animation must be for a single node, which is the
///                local entity. If the animation contains more than a single node index, the
///                animation will be silently dropped.
/// - `sampler_storage`: `AssetStorage` for all `Sampler`s
/// - `samplers`: the active sampler sets
/// - `targets`: Target components, used to retrieve the rest pose before animation starts.
/// - `remove`: all entities pushed here will have the control object removed at the end of the system execution
/// - `next_id`: next id to use for the animation control id
///
/// ##
///
/// Optionally returns a new `ControlState` for the animation. This will be the new state of the
/// control object.
fn process_animation_control<T>(
    entity: &Entity,
    animation: &Animation<T>,
    control: &mut AnimationControl<T>,
    hierarchy: Option<&AnimationHierarchy<T>>,
    sampler_storage: &AssetStorage<Sampler<T::Primitive>>,
    samplers: &mut WriteStorage<SamplerControlSet<T>>,
    targets: &ReadStorage<T>,
    remove: &mut bool,
    next_id: &mut u64,
) -> Option<ControlState>
where
    T: AnimationSampling + Component,
{
    // Checking hierarchy
    let h_fallback = AnimationHierarchy::new_single(animation.nodes[0].0, *entity);
    let hierarchy = match hierarchy {
        Some(h) => h,
        None => if only_one_index(&animation.nodes) {
            &h_fallback
        } else {
            error!(
                "Animation control which target multiple nodes without a hierarchy detected, dropping"
            );
            *remove = true;
            return None;
        },
    };
    match (&control.state, &control.command) {
        // Check for aborted or done animation
        (_, &AnimationCommand::Abort) | (&ControlState::Abort, _) | (&ControlState::Done, _) => {
            // signal samplers to abort, and remove control object if all samplers are done and removed
            if check_and_terminate_animation(control.id, hierarchy, samplers) {
                *remove = true;
            }
            Some(ControlState::Abort)
        }

        // Animation was just requested, start it
        // We ignore the command here because we need the animation to be
        // started before we can pause it, and to avoid a lot of checks for
        // abort. The command will be processed next frame.
        (&ControlState::Requested, &AnimationCommand::Start) => {
            control.id = *next_id;
            *next_id += 1;
            if start_animation(
                animation,
                sampler_storage,
                control,
                hierarchy,
                samplers,
                targets,
            ) {
                Some(ControlState::Running(Duration::from_secs(0)))
            } else {
                None // Try again next frame, might just be that samplers haven't finished loading
            }
        }

        // If pause was requested on a running animation, pause it
        (&ControlState::Running(..), &AnimationCommand::Pause) => {
            pause_animation(control.id, hierarchy, samplers);
            Some(ControlState::Paused(Duration::from_secs(0)))
        }

        // If start was requested on a paused animation, unpause it
        (&ControlState::Paused(_), &AnimationCommand::Start) => {
            unpause_animation(control.id, hierarchy, samplers);
            Some(ControlState::Running(Duration::from_secs(0)))
        }

        (&ControlState::Running(..), &AnimationCommand::Step(ref dir)) => {
            step_animation(control.id, hierarchy, samplers, sampler_storage, dir);
            None
        }

        (&ControlState::Running(..), &AnimationCommand::SetInputValue(value)) => {
            set_animation_input(control.id, hierarchy, samplers, value);
            None
        }

        (&ControlState::Running(..), &AnimationCommand::SetBlendWeights(ref weights)) => {
            set_blend_weights(control.id, hierarchy, samplers, weights);
            None
        }

        // check for finished/aborted animations, wait for samplers to signal done,
        // then remove control objects
        (&ControlState::Running(..), _) => {
            if check_termination(control.id, hierarchy, &samplers) {
                // Do termination
                for (_, node_entity) in &hierarchy.nodes {
                    let empty = {
                        let mut sampler = samplers.get_mut(*node_entity).unwrap();
                        sampler.clear(control.id);
                        sampler.is_empty()
                    };
                    if empty {
                        samplers.remove(*node_entity);
                    }
                }
                *remove = true;
            } else {
                update_animation_rate(control.id, hierarchy, samplers, control.rate_multiplier);
            }
            None
        }

        _ => None,
    }
}

/// Process animation creation request.
/// Will build `SamplerControlSet`s for the `AnimationHierarchy` given, based on the `Sampler`s in
/// the given `Animation`.
///
/// ## Parameters
///
/// - `animation`: the animation to start
/// - `sampler_storage`: all samplers
/// - `control`: the control object for the animation instance
/// - `hierarchy`: the animation node hierarchy for the entity hierarchy the animation instance is active for
/// - `samplers`: the active sampler sets
/// - `targets`: Target components, used to retrieve the rest pose before animation starts.
///
/// ## Returns
///
/// True if the animation was started, false if it wasn't.
fn start_animation<T>(
    animation: &Animation<T>,
    sampler_storage: &AssetStorage<Sampler<T::Primitive>>,
    control: &AnimationControl<T>,
    hierarchy: &AnimationHierarchy<T>,
    samplers: &mut WriteStorage<SamplerControlSet<T>>,
    targets: &ReadStorage<T>, // for rest state
) -> bool
where
    T: AnimationSampling + Component,
{
    // check that hierarchy is valid, and all samplers exist
    if animation
        .nodes
        .iter()
        .any(|&(ref node_index, _, ref sampler_handle)| {
            hierarchy.nodes.get(node_index).is_none()
                || sampler_storage.get(sampler_handle).is_none()
        }) {
        return false;
    }

    // setup sampler tree
    for &(ref node_index, ref channel, ref sampler_handle) in &animation.nodes {
        let node_entity = hierarchy.nodes.get(node_index).unwrap();
        let component = targets.get(*node_entity).unwrap();
        let sampler_control = SamplerControl::<T> {
            control_id: control.id,
            channel: channel.clone(),
            state: ControlState::Requested,
            sampler: sampler_handle.clone(),
            end: control.end.clone(),
            after: component.current_sample(channel),
            rate_multiplier: control.rate_multiplier,
            blend_weight: 1.0,
        };
        let add = if let Some(ref mut set) = samplers.get_mut(*node_entity) {
            set.add_control(sampler_control);
            None
        } else {
            Some(sampler_control)
        };
        if let Some(sampler_control) = add {
            let mut set = SamplerControlSet::default();
            set.add_control(sampler_control);
            samplers.insert(*node_entity, set);
        }
    }
    true
}

fn pause_animation<T>(
    control_id: u64,
    hierarchy: &AnimationHierarchy<T>,
    samplers: &mut WriteStorage<SamplerControlSet<T>>,
) where
    T: AnimationSampling,
{
    for (_, node_entity) in &hierarchy.nodes {
        if let Some(ref mut s) = samplers.get_mut(*node_entity) {
            s.pause(control_id);
        }
    }
}

fn unpause_animation<T>(
    control_id: u64,
    hierarchy: &AnimationHierarchy<T>,
    samplers: &mut WriteStorage<SamplerControlSet<T>>,
) where
    T: AnimationSampling,
{
    for (_, node_entity) in &hierarchy.nodes {
        if let Some(ref mut s) = samplers.get_mut(*node_entity) {
            s.unpause(control_id);
        }
    }
}

fn step_animation<T>(
    control_id: u64,
    hierarchy: &AnimationHierarchy<T>,
    controls: &mut WriteStorage<SamplerControlSet<T>>,
    sampler_storage: &AssetStorage<Sampler<T::Primitive>>,
    direction: &StepDirection,
) where
    T: AnimationSampling,
{
    for (_, node_entity) in &hierarchy.nodes {
        if let Some(ref mut s) = controls.get_mut(*node_entity) {
            s.step(control_id, sampler_storage, direction);
        }
    }
}

fn set_animation_input<T>(
    control_id: u64,
    hierarchy: &AnimationHierarchy<T>,
    controls: &mut WriteStorage<SamplerControlSet<T>>,
    input: f32,
) where
    T: AnimationSampling,
{
    for (_, node_entity) in &hierarchy.nodes {
        if let Some(ref mut s) = controls.get_mut(*node_entity) {
            s.set_input(control_id, input);
        }
    }
}

fn set_blend_weights<T>(
    control_id: u64,
    hierarchy: &AnimationHierarchy<T>,
    controls: &mut WriteStorage<SamplerControlSet<T>>,
    weights: &Vec<(usize, T::Channel, f32)>,
) where
    T: AnimationSampling,
{
    for &(node_index, ref channel, weight) in weights {
        if let Some(node_entity) = hierarchy.nodes.get(&node_index) {
            if let Some(ref mut s) = controls.get_mut(*node_entity) {
                s.set_blend_weight(control_id, channel, weight);
            }
        }
    }
}

fn update_animation_rate<T>(
    control_id: u64,
    hierarchy: &AnimationHierarchy<T>,
    samplers: &mut WriteStorage<SamplerControlSet<T>>,
    rate_multiplier: f32,
) where
    T: AnimationSampling,
{
    for (_, node_entity) in &hierarchy.nodes {
        if let Some(ref mut s) = samplers.get_mut(*node_entity) {
            s.set_rate_multiplier(control_id, rate_multiplier);
        }
    }
}

/// Check if all nodes in an `AnimationHierarchy` are ready for termination, if so remove all
/// `SamplerControlSet`s for the hierarchy, if not request termination on all sampler controls
fn check_and_terminate_animation<T>(
    control_id: u64,
    hierarchy: &AnimationHierarchy<T>,
    samplers: &mut WriteStorage<SamplerControlSet<T>>,
) -> bool
where
    T: AnimationSampling,
{
    // Check for termination
    if check_termination(control_id, hierarchy, &samplers) {
        // Do termination
        for (_, node_entity) in &hierarchy.nodes {
            let empty = {
                let mut sampler = samplers.get_mut(*node_entity).unwrap();
                sampler.clear(control_id);
                sampler.is_empty()
            };
            if empty {
                samplers.remove(*node_entity);
            }
        }
        true
    } else {
        // Request termination of samplers
        for (_, node_entity) in &hierarchy.nodes {
            if let Some(ref mut s) = samplers.get_mut(*node_entity) {
                s.abort(control_id);
            }
        }
        false
    }
}

/// Check if all nodes in an `AnimationHierarcy` are ready for termination.
fn check_termination<T>(
    control_id: u64,
    hierarchy: &AnimationHierarchy<T>,
    samplers: &WriteStorage<SamplerControlSet<T>>,
) -> bool
where
    T: AnimationSampling,
{
    hierarchy
        .nodes
        .iter()
        .flat_map(|(_, node_entity)| samplers.get(*node_entity))
        .all(|s| s.check_termination(control_id))
}
