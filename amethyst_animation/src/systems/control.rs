use std::marker;
use std::time::Duration;

use amethyst_assets::{AssetStorage, Handle};
use fnv::FnvHashMap;
use minterpolate::InterpolationPrimitive;
use specs::{Component, Entities, Entity, Fetch, Join, ReadStorage, System, WriteStorage};

use resources::{Animation, AnimationCommand, AnimationControl, AnimationHierarchy,
                AnimationSampling, ControlState, Sampler, SamplerControl, SamplerControlSet};

/// System for setting up animations, should run before `SamplerInterpolationSystem`.
///
/// Will process all active `AnimationControl` + `AnimationHierarchy`, and do processing of the
/// animations they describe. If an animation only targets a single node/entity, there is no need
/// for `AnimationHierarchy`.
#[derive(Default)]
pub struct AnimationControlSystem<T> {
    m: marker::PhantomData<T>,
}

impl<T> AnimationControlSystem<T> {
    pub fn new() -> Self {
        Self {
            m: marker::PhantomData,
        }
    }
}

impl<'a, T> System<'a> for AnimationControlSystem<T>
where
    T: AnimationSampling + Component,
{
    type SystemData = (
        Entities<'a>,
        Fetch<'a, AssetStorage<Animation<T>>>,
        Fetch<'a, AssetStorage<Sampler<T::Primitive>>>,
        WriteStorage<'a, AnimationControl<T>>,
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
        let mut remove = Vec::default();
        for (entity, control) in (&*entities, &mut controls).join() {
            if let Some(state) = animation_storage
                .get(&control.animation)
                .and_then(|animation| {
                    process_animation_control(
                        &entity,
                        animation,
                        control,
                        hierarchies.get(entity),
                        &*sampler_storage,
                        &mut samplers,
                        &transforms,
                        &mut remove,
                    )
                }) {
                control.state = state;
            }
        }

        for entity in remove {
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
/// - `now`: we want all animations and samplers to be synchronized, so we only supply an `Instant`
///
/// ##
///
/// Optionally returns a new `ControlState` for the animation. This will be the new state of the
/// control object.
fn process_animation_control<T>(
    entity: &Entity,
    animation: &Animation<T>,
    control: &AnimationControl<T>,
    hierarchy: Option<&AnimationHierarchy<T>>,
    sampler_storage: &AssetStorage<Sampler<T::Primitive>>,
    samplers: &mut WriteStorage<SamplerControlSet<T>>,
    targets: &ReadStorage<T>,
    remove: &mut Vec<Entity>,
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
            eprintln!(
                "Animation control which target multiple nodes without a hierarchy detected, dropping"
            );
            remove.push(*entity);
            return None;
        },
    };
    match (&control.state, &control.command) {
        // Check for aborted or done animation
        (_, &AnimationCommand::Abort) | (&ControlState::Abort, _) | (&ControlState::Done, _) => {
            // signal samplers to abort, and remove control object if all samplers are done and removed
            if check_and_terminate_animation(hierarchy, samplers) {
                remove.push(*entity);
            }
            Some(ControlState::Abort)
        }

        // Animation was just requested, start it
        // We ignore the command here because we need the animation to be
        // started before we can pause it, and to avoid a lot of checks for
        // abort. The command will be processed next frame.
        (&ControlState::Requested, _) => {
            if request_animation(
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
            pause_animation(hierarchy, samplers);
            Some(ControlState::Paused(Duration::from_secs(0)))
        }

        // If start was requested on a paused animation, unpause it
        (&ControlState::Paused(_), &AnimationCommand::Start) => {
            unpause_animation(hierarchy, samplers);
            Some(ControlState::Running(Duration::from_secs(0)))
        }

        // check for finished/aborted animations, wait for samplers to signal done,
        // then remove control objects
        (&ControlState::Running(..), _) => {
            if check_termination(hierarchy, &samplers) {
                // Do termination
                for (_, node_entity) in &hierarchy.nodes {
                    samplers.remove(*node_entity);
                }
                remove.push(*entity);
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
fn request_animation<T>(
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
            channel: channel.clone(),
            state: ControlState::Requested,
            sampler: sampler_handle.clone(),
            end: control.end.clone(),
            after: get_after(channel, component),
        };
        let add = if let Some(ref mut set) = samplers.get_mut(*node_entity) {
            add_to_set(channel, set, sampler_control);
            None
        } else {
            Some(sampler_control)
        };
        if let Some(sampler_control) = add {
            let mut set = SamplerControlSet {
                samplers: FnvHashMap::default(),
            };
            add_to_set(channel, &mut set, sampler_control);
            samplers.insert(*node_entity, set);
        }
    }
    true
}

fn add_to_set<T>(
    channel: &T::Channel,
    control_set: &mut SamplerControlSet<T>,
    control: SamplerControl<T>,
) where
    T: AnimationSampling,
{
    control_set.samplers.insert(channel.clone(), control);
}

fn get_after<T>(channel: &T::Channel, component: &T) -> T::Primitive
where
    T: AnimationSampling,
{
    component.current_sample(channel)
}

fn pause_animation<T>(
    hierarchy: &AnimationHierarchy<T>,
    samplers: &mut WriteStorage<SamplerControlSet<T>>,
) where
    T: AnimationSampling,
{
    for (_, node_entity) in &hierarchy.nodes {
        match samplers.get_mut(*node_entity) {
            Some(ref mut s) => do_control_set_pause(s),
            _ => (),
        }
    }
}

fn unpause_animation<T>(
    hierarchy: &AnimationHierarchy<T>,
    samplers: &mut WriteStorage<SamplerControlSet<T>>,
) where
    T: AnimationSampling,
{
    for (_, node_entity) in &hierarchy.nodes {
        match samplers.get_mut(*node_entity) {
            Some(ref mut s) => do_control_set_unpause(s),
            _ => (),
        }
    }
}

/// Check if all nodes in an `AnimationHierarchy` are ready for termination, if so remove all
/// `SamplerControlSet`s for the hierarchy, if not request termination on all sampler controls
fn check_and_terminate_animation<T>(
    hierarchy: &AnimationHierarchy<T>,
    samplers: &mut WriteStorage<SamplerControlSet<T>>,
) -> bool
where
    T: AnimationSampling,
{
    // Check for termination
    if check_termination(hierarchy, &samplers) {
        // Do termination
        for (_, node_entity) in &hierarchy.nodes {
            samplers.remove(*node_entity);
        }
        true
    } else {
        // Request termination of samplers
        for (_, node_entity) in &hierarchy.nodes {
            match samplers.get_mut(*node_entity) {
                Some(ref mut s) => do_control_set_termination(s),
                _ => (),
            }
        }
        false
    }
}

/// Check if all nodes in an `AnimationHierarcy` are ready for termination.
fn check_termination<T>(
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
        .all(check_control_set_termination)
}

/// Request termination of `SamplerControlSet`
fn do_control_set_termination<T>(control_set: &mut SamplerControlSet<T>)
where
    T: AnimationSampling,
{
    for sampler in control_set
        .samplers
        .values_mut()
        .filter(|t| t.state != ControlState::Done)
    {
        sampler.state = ControlState::Abort;
    }
}

/// Pause a `SamplerControlSet`
fn do_control_set_pause<T>(control_set: &mut SamplerControlSet<T>)
where
    T: AnimationSampling,
{
    for sampler in control_set.samplers.values_mut() {
        sampler.state = match sampler.state {
            ControlState::Running(dur) => ControlState::Paused(dur),
            _ => ControlState::Paused(Duration::from_secs(0)),
        }
    }
}

/// Unpause a `SamplerControlSet`
fn do_control_set_unpause<T>(control_set: &mut SamplerControlSet<T>)
where
    T: AnimationSampling,
{
    for sampler in control_set.samplers.values_mut() {
        if let ControlState::Paused(dur) = sampler.state {
            sampler.state = ControlState::Running(dur);
        }
    }
}

/// Check if a control set can be terminated
fn check_control_set_termination<T>(control_set: &SamplerControlSet<T>) -> bool
where
    T: AnimationSampling,
{
    control_set
        .samplers
        .values()
        .all(|t| t.state == ControlState::Done || t.state == ControlState::Requested)
}
