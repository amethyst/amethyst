use std::{hash::Hash, marker, time::Duration};

use fnv::FnvHashMap;
use log::error;
use minterpolate::InterpolationPrimitive;

use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    specs::prelude::{
        Component, Entities, Entity, Join, Read, ReadStorage, Resources, System, SystemData,
        WriteStorage,
    },
    timing::secs_to_duration,
};

use crate::resources::{
    Animation, AnimationCommand, AnimationControl, AnimationControlSet, AnimationHierarchy,
    AnimationSampling, AnimationSet, ApplyData, ControlState, DeferStartRelation, RestState,
    Sampler, SamplerControl, SamplerControlSet, StepDirection,
};

/// System for setting up animations, should run before `SamplerInterpolationSystem`.
///
/// Will process all active `AnimationControl` + `AnimationHierarchy`, and do processing of the
/// animations they describe. If an animation only targets a single node/entity, there is no need
/// for `AnimationHierarchy`.
///
/// ### Type parameters:
///
/// - `I`: identifier type for running animations, only one animation can be run at the same time
///        with the same id
/// - `T`: the component type that the animation should be applied to
#[derive(Default)]
pub struct AnimationControlSystem<I, T>
where
    I: Eq + Hash,
{
    m: marker::PhantomData<(I, T)>,
    next_id: u64,
    remove_ids: Vec<I>,
    state_set: FnvHashMap<I, f32>,
    deferred_start: Vec<(I, f32)>,
}

impl<I, T> AnimationControlSystem<I, T>
where
    I: Eq + Hash,
{
    /// Creates a new `AnimationControlSystem`
    pub fn new() -> Self {
        AnimationControlSystem {
            m: marker::PhantomData,
            next_id: 1,
            remove_ids: Vec::default(),
            state_set: FnvHashMap::default(),
            deferred_start: Vec::default(),
        }
    }
}

impl<'a, I, T> System<'a> for AnimationControlSystem<I, T>
where
    I: PartialEq + Eq + Hash + Copy + Send + Sync + 'static,
    T: AnimationSampling + Component + Clone,
{
    type SystemData = (
        Entities<'a>,
        Read<'a, AssetStorage<Animation<T>>>,
        Read<'a, AssetStorage<Sampler<T::Primitive>>>,
        WriteStorage<'a, AnimationControlSet<I, T>>,
        WriteStorage<'a, SamplerControlSet<T>>,
        ReadStorage<'a, AnimationHierarchy<T>>,
        ReadStorage<'a, T>,
        WriteStorage<'a, RestState<T>>,
        <T as ApplyData<'a>>::ApplyData,
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
            mut rest_states,
            apply_data,
        ) = data;
        let mut remove_sets = Vec::default();
        for (entity, control_set) in (&*entities, &mut controls).join() {
            self.remove_ids.clear();
            self.state_set.clear();
            let hierarchy = hierarchies.get(entity);
            for &mut (ref id, ref mut control) in control_set.animations.iter_mut() {
                let mut remove = false;
                if let Some(state) =
                    animation_storage
                        .get(&control.animation)
                        .and_then(|animation| {
                            process_animation_control(
                                &entity,
                                animation,
                                control,
                                hierarchy,
                                &*sampler_storage,
                                &mut samplers,
                                &mut rest_states,
                                &transforms,
                                &mut remove,
                                &mut self.next_id,
                                &apply_data,
                            )
                        })
                {
                    control.state = state;
                }
                if let AnimationCommand::Step(_) = control.command {
                    control.command = AnimationCommand::Start;
                }
                if let AnimationCommand::SetInputValue(_) = control.command {
                    control.command = AnimationCommand::Start;
                }
                if remove {
                    self.remove_ids.push(*id);
                } else {
                    self.state_set.insert(
                        *id,
                        get_running_duration(&entity, control, hierarchies.get(entity), &samplers),
                    );
                }
            }
            for deferred_animation in &control_set.deferred_animations {
                self.state_set.insert(deferred_animation.animation_id, -1.0);
            }
            self.deferred_start.clear();
            for deferred_animation in &control_set.deferred_animations {
                let (start, start_dur) =
                    if let Some(dur) = self.state_set.get(&deferred_animation.relation.0) {
                        if *dur < 0. {
                            (false, 0.)
                        } else if let DeferStartRelation::Start(start_dur) =
                            deferred_animation.relation.1
                        {
                            let remain_dur = dur - start_dur;
                            (remain_dur >= 0., remain_dur)
                        } else {
                            (false, 0.)
                        }
                    } else {
                        (true, 0.)
                    };
                if start {
                    self.deferred_start
                        .push((deferred_animation.animation_id, start_dur));
                    self.state_set
                        .insert(deferred_animation.animation_id, start_dur);
                }
            }
            let mut next_id = self.next_id;
            for &(id, start_dur) in &self.deferred_start {
                let index = control_set
                    .deferred_animations
                    .iter()
                    .position(|a| a.animation_id == id)
                    .expect("Unreachable: Id of current `deferred_start` was taken from previous loop over `deferred_animations`");

                let mut def = control_set.deferred_animations.remove(index);
                def.control.state = ControlState::Deferred(secs_to_duration(start_dur));
                def.control.command = AnimationCommand::Start;
                let mut remove = false;
                if let Some(state) =
                    animation_storage
                        .get(&def.control.animation)
                        .and_then(|animation| {
                            process_animation_control(
                                &entity,
                                animation,
                                &mut def.control,
                                hierarchy,
                                &*sampler_storage,
                                &mut samplers,
                                &mut rest_states,
                                &transforms,
                                &mut remove,
                                &mut next_id,
                                &apply_data,
                            )
                        })
                {
                    def.control.state = state;
                }
                control_set.insert(id, def.control);
            }
            self.next_id = next_id;
            for id in &self.remove_ids {
                control_set.remove(*id);
                if control_set.is_empty() {
                    remove_sets.push(entity);
                }
            }
        }

        for entity in remove_sets {
            controls.remove(entity);
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        ReadStorage::<AnimationSet<I, T>>::setup(res);
    }
}

fn get_running_duration<T>(
    entity: &Entity,
    control: &AnimationControl<T>,
    hierarchy: Option<&AnimationHierarchy<T>>,
    samplers: &WriteStorage<'_, SamplerControlSet<T>>,
) -> f32
where
    T: AnimationSampling,
{
    match control.state {
        ControlState::Running(_) => find_max_duration(
            control.id,
            samplers.get(
                *hierarchy
                    .and_then(|h| h.nodes.values().next())
                    .unwrap_or(entity),
            ),
        ),
        _ => -1.0,
    }
}

fn find_max_duration<T>(control_id: u64, samplers: Option<&SamplerControlSet<T>>) -> f32
where
    T: AnimationSampling,
{
    samplers
        .and_then(|set| set.get_running_duration(control_id))
        .unwrap_or(0.)
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
    samplers: &mut WriteStorage<'_, SamplerControlSet<T>>,
    rest_states: &mut WriteStorage<'_, RestState<T>>,
    targets: &ReadStorage<'_, T>,
    remove: &mut bool,
    next_id: &mut u64,
    apply_data: &<T as ApplyData<'_>>::ApplyData,
) -> Option<ControlState>
where
    T: AnimationSampling + Component + Clone,
{
    // Checking hierarchy
    let h_fallback = AnimationHierarchy::new_single(animation.nodes[0].0, *entity);
    let hierarchy = match hierarchy {
        Some(h) => h,
        None => {
            if only_one_index(&animation.nodes) {
                &h_fallback
            } else {
                error!(
                "Animation control which target multiple nodes without a hierarchy detected, dropping"
            );
                *remove = true;
                return None;
            }
        }
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
                rest_states,
                targets,
                apply_data,
            ) {
                Some(ControlState::Running(Duration::from_secs(0)))
            } else {
                None // Try again next frame, might just be that samplers haven't finished loading
            }
        }

        (&ControlState::Deferred(..), &AnimationCommand::Start) => {
            control.id = *next_id;
            *next_id += 1;
            if start_animation(
                animation,
                sampler_storage,
                control,
                hierarchy,
                samplers,
                rest_states,
                targets,
                apply_data,
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
                for node_entity in hierarchy.nodes.values() {
                    let empty = samplers
                        .get_mut(*node_entity)
                        .map(|sampler| {
                            sampler.clear(control.id);
                            sampler.is_empty()
                        })
                        .unwrap_or(false);
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
    samplers: &mut WriteStorage<'_, SamplerControlSet<T>>,
    rest_states: &mut WriteStorage<'_, RestState<T>>,
    targets: &ReadStorage<'_, T>, // for rest state
    apply_data: &<T as ApplyData<'_>>::ApplyData,
) -> bool
where
    T: AnimationSampling + Component + Clone,
{
    // check that hierarchy is valid, and all samplers exist
    if animation
        .nodes
        .iter()
        .any(|&(ref node_index, _, ref sampler_handle)| {
            !hierarchy.nodes.contains_key(node_index)
                || sampler_storage.get(sampler_handle).is_none()
        })
    {
        return false;
    }

    hierarchy.rest_state(|entity| targets.get(entity).cloned(), rest_states);

    let start_state = if let ControlState::Deferred(dur) = control.state {
        ControlState::Deferred(dur)
    } else {
        ControlState::Requested
    };

    // setup sampler tree
    for &(ref node_index, ref channel, ref sampler_handle) in &animation.nodes {
        let node_entity = hierarchy.nodes.get(node_index).expect(
            "Unreachable: Existence of all nodes are checked in validation of hierarchy above",
        );
        let component = rest_states
            .get(*node_entity)
            .map(|r| r.state())
            .or_else(|| targets.get(*node_entity))
            .expect(
                "Unreachable: Existence of all nodes are checked in validation of hierarchy above",
            );
        let sampler_control = SamplerControl::<T> {
            control_id: control.id,
            channel: channel.clone(),
            state: start_state.clone(),
            sampler: sampler_handle.clone(),
            end: control.end.clone(),
            after: component.current_sample(channel, apply_data),
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
            if let Err(err) = samplers.insert(*node_entity, set) {
                error!(
                    "Failed creating SamplerControl for AnimationHierarchy because: {}",
                    err
                );
            }
        }
    }
    true
}

fn pause_animation<T>(
    control_id: u64,
    hierarchy: &AnimationHierarchy<T>,
    samplers: &mut WriteStorage<'_, SamplerControlSet<T>>,
) where
    T: AnimationSampling,
{
    for node_entity in hierarchy.nodes.values() {
        if let Some(ref mut s) = samplers.get_mut(*node_entity) {
            s.pause(control_id);
        }
    }
}

fn unpause_animation<T>(
    control_id: u64,
    hierarchy: &AnimationHierarchy<T>,
    samplers: &mut WriteStorage<'_, SamplerControlSet<T>>,
) where
    T: AnimationSampling,
{
    for node_entity in hierarchy.nodes.values() {
        if let Some(ref mut s) = samplers.get_mut(*node_entity) {
            s.unpause(control_id);
        }
    }
}

fn step_animation<T>(
    control_id: u64,
    hierarchy: &AnimationHierarchy<T>,
    controls: &mut WriteStorage<'_, SamplerControlSet<T>>,
    sampler_storage: &AssetStorage<Sampler<T::Primitive>>,
    direction: &StepDirection,
) where
    T: AnimationSampling,
{
    for node_entity in hierarchy.nodes.values() {
        if let Some(ref mut s) = controls.get_mut(*node_entity) {
            s.step(control_id, sampler_storage, direction);
        }
    }
}

fn set_animation_input<T>(
    control_id: u64,
    hierarchy: &AnimationHierarchy<T>,
    controls: &mut WriteStorage<'_, SamplerControlSet<T>>,
    input: f32,
) where
    T: AnimationSampling,
{
    for node_entity in hierarchy.nodes.values() {
        if let Some(ref mut s) = controls.get_mut(*node_entity) {
            s.set_input(control_id, input);
        }
    }
}

fn set_blend_weights<T>(
    control_id: u64,
    hierarchy: &AnimationHierarchy<T>,
    controls: &mut WriteStorage<'_, SamplerControlSet<T>>,
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
    samplers: &mut WriteStorage<'_, SamplerControlSet<T>>,
    rate_multiplier: f32,
) where
    T: AnimationSampling,
{
    for node_entity in hierarchy.nodes.values() {
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
    samplers: &mut WriteStorage<'_, SamplerControlSet<T>>,
) -> bool
where
    T: AnimationSampling,
{
    // Check for termination
    if check_termination(control_id, hierarchy, &samplers) {
        // Do termination
        for node_entity in hierarchy.nodes.values() {
            let empty = samplers
                .get_mut(*node_entity)
                .map(|sampler| {
                    sampler.clear(control_id);
                    sampler.is_empty()
                })
                .unwrap_or(false);
            if empty {
                samplers.remove(*node_entity);
            }
        }
        true
    } else {
        // Request termination of samplers
        for node_entity in hierarchy.nodes.values() {
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
    samplers: &WriteStorage<'_, SamplerControlSet<T>>,
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
