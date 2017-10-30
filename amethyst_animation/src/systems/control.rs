use std::time::Duration;

use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::LocalTransform;
use specs::{Entities, Entity, Fetch, Join, ReadStorage, System, WriteStorage};

use resources::{Animation, AnimationCommand, AnimationControl, AnimationHierarchy,
                AnimationOutput, ControlState, RestState, Sampler, SamplerControl,
                SamplerControlSet};

/// System for setting up animations, should run before `SamplerInterpolationSystem`.
///
/// Will process all active `AnimationControl` + `AnimationHierarchy`, and do processing of the
/// animations they describe. If an animation only targets a single node/entity, there is no need
/// for `AnimationHierarchy`.
#[derive(Default)]
pub struct AnimationControlSystem;

impl AnimationControlSystem {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a> System<'a> for AnimationControlSystem {
    type SystemData = (
        Entities<'a>,
        Fetch<'a, AssetStorage<Animation>>,
        Fetch<'a, AssetStorage<Sampler>>,
        WriteStorage<'a, AnimationControl>,
        WriteStorage<'a, SamplerControlSet>,
        ReadStorage<'a, AnimationHierarchy>,
        ReadStorage<'a, LocalTransform>,
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
fn only_one_index(nodes: &[(usize, Handle<Sampler>)]) -> bool {
    if nodes.is_empty() {
        true
    } else {
        let first = nodes[0].0;
        nodes.iter().all(|&(ref i, _)| *i == first)
    }
}

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::fnv::FnvHashMap::default();
         $( map.insert($key, $val); )*
         map
    }}
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
/// - `transforms`: `LocalTransform`s, used to retrieve the rest pose before animation starts.
/// - `remove`: all entities pushed here will have the control object removed at the end of the system execution
/// - `now`: we want all animations and samplers to be synchronized, so we only supply an `Instant`
///
/// ##
///
/// Optionally returns a new `ControlState` for the animation. This will be the new state of the
/// control object.
fn process_animation_control(
    entity: &Entity,
    animation: &Animation,
    control: &AnimationControl,
    hierarchy: Option<&AnimationHierarchy>,
    sampler_storage: &AssetStorage<Sampler>,
    samplers: &mut WriteStorage<SamplerControlSet>,
    transforms: &ReadStorage<LocalTransform>,
    remove: &mut Vec<Entity>,
) -> Option<ControlState> {
    // Checking hierarchy
    let h_fallback = AnimationHierarchy {
        nodes: hashmap![animation.nodes[0].0 => *entity],
    };
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
                transforms,
            ) {
                Some(ControlState::Running(Duration::from_secs(0)))
            } else {
                None // Try again next frame, might just be that samplers haven't
                     // finished loading
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
/// - `transforms`: `LocalTransform`s, used to retrieve the rest pose before animation starts.
///
/// ## Returns
///
/// True if the animation was started, false if it wasn't.
fn request_animation(
    animation: &Animation,
    sampler_storage: &AssetStorage<Sampler>,
    control: &AnimationControl,
    hierarchy: &AnimationHierarchy,
    samplers: &mut WriteStorage<SamplerControlSet>,
    transforms: &ReadStorage<LocalTransform>, // for rest state
) -> bool {
    // check that hierarchy is valid, and all samplers exist
    if animation
        .nodes
        .iter()
        .any(|&(ref node_index, ref sampler_handle)| {
            hierarchy.nodes.get(node_index).is_none()
                || sampler_storage.get(sampler_handle).is_none()
        }) {
        return false;
    }

    // setup sampler tree
    for &(ref node_index, ref sampler_handle) in &animation.nodes {
        let node_entity = hierarchy.nodes.get(node_index).unwrap();
        let sampler = sampler_storage.get(sampler_handle).unwrap();
        let transform = transforms.get(*node_entity).unwrap();
        let sampler_control = SamplerControl {
            sampler: sampler_handle.clone(),
            state: ControlState::Requested,
            end: control.end.clone(),
            after: get_after(sampler, transform),
        };
        let mut set = samplers
            .get(*node_entity)
            .cloned()
            .unwrap_or(SamplerControlSet::default());
        add_to_set(sampler, &mut set, sampler_control);
        samplers.insert(*node_entity, set);
    }
    true
}

fn add_to_set(sampler: &Sampler, control_set: &mut SamplerControlSet, control: SamplerControl) {
    match sampler.output {
        AnimationOutput::Translation(_) => control_set.translation = Some(control),
        AnimationOutput::Rotation(_) => control_set.rotation = Some(control),
        AnimationOutput::Scale(_) => control_set.scale = Some(control),
    }
}

fn get_after(sampler: &Sampler, transform: &LocalTransform) -> RestState {
    match sampler.output {
        AnimationOutput::Translation(_) => RestState::Translation(transform.translation.into()),
        AnimationOutput::Rotation(_) => RestState::Rotation(transform.rotation.into()),
        AnimationOutput::Scale(_) => RestState::Scale(transform.scale.into()),
    }
}

fn pause_animation(hierarchy: &AnimationHierarchy, samplers: &mut WriteStorage<SamplerControlSet>) {
    for (_, node_entity) in &hierarchy.nodes {
        match samplers.get_mut(*node_entity) {
            Some(ref mut s) => do_control_set_pause(s),
            _ => (),
        }
    }
}

fn unpause_animation(
    hierarchy: &AnimationHierarchy,
    samplers: &mut WriteStorage<SamplerControlSet>,
) {
    for (_, node_entity) in &hierarchy.nodes {
        match samplers.get_mut(*node_entity) {
            Some(ref mut s) => do_control_set_unpause(s),
            _ => (),
        }
    }
}

/// Check if all nodes in an `AnimationHierarchy` are ready for termination, if so remove all
/// `SamplerControlSet`s for the hierarchy, if not request termination on all sampler controls
fn check_and_terminate_animation(
    hierarchy: &AnimationHierarchy,
    samplers: &mut WriteStorage<SamplerControlSet>,
) -> bool {
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
fn check_termination(
    hierarchy: &AnimationHierarchy,
    samplers: &WriteStorage<SamplerControlSet>,
) -> bool {
    hierarchy
        .nodes
        .iter()
        .flat_map(|(_, node_entity)| samplers.get(*node_entity))
        .all(check_control_set_termination)
}

/// Request termination of `SamplerControlSet`
fn do_control_set_termination(control_set: &mut SamplerControlSet) {
    if let Some(ref mut t) = control_set.translation {
        if t.state != ControlState::Done {
            t.state = ControlState::Abort;
        }
    }
    if let Some(ref mut r) = control_set.rotation {
        if r.state != ControlState::Done {
            r.state = ControlState::Abort;
        }
    }
    if let Some(ref mut s) = control_set.scale {
        if s.state != ControlState::Done {
            s.state = ControlState::Abort;
        }
    }
}

/// Pause a `SamplerControlSet`
fn do_control_set_pause(control_set: &mut SamplerControlSet) {
    if let Some(ref mut t) = control_set.translation {
        t.state = match t.state {
            ControlState::Running(dur) => ControlState::Paused(dur),
            _ => ControlState::Paused(Duration::from_secs(0)),
        };
    }
    if let Some(ref mut r) = control_set.rotation {
        r.state = match r.state {
            ControlState::Running(dur) => ControlState::Paused(dur),
            _ => ControlState::Paused(Duration::from_secs(0)),
        };
    }
    if let Some(ref mut s) = control_set.scale {
        s.state = match s.state {
            ControlState::Running(dur) => ControlState::Paused(dur),
            _ => ControlState::Paused(Duration::from_secs(0)),
        };
    }
}

/// Unpause a `SamplerControlSet`
fn do_control_set_unpause(control_set: &mut SamplerControlSet) {
    if let Some(ref mut t) = control_set.translation {
        t.state = match t.state {
            ControlState::Paused(dur) => ControlState::Running(dur),
            ref s => s.clone(),
        };
    }
    if let Some(ref mut r) = control_set.rotation {
        r.state = match r.state {
            ControlState::Paused(dur) => ControlState::Running(dur),
            ref s => s.clone(),
        };
    }
    if let Some(ref mut s) = control_set.scale {
        s.state = match s.state {
            ControlState::Paused(dur) => ControlState::Running(dur),
            ref s => s.clone(),
        };
    }
}

/// Check if a control set can be terminated
fn check_control_set_termination(control_set: &SamplerControlSet) -> bool {
    control_set
        .translation
        .as_ref()
        .map(|t| {
            t.state == ControlState::Done || t.state == ControlState::Requested
        })
        .unwrap_or(true)
        && control_set
            .rotation
            .as_ref()
            .map(|r| {
                r.state == ControlState::Done || r.state == ControlState::Requested
            })
            .unwrap_or(true)
        && control_set
            .scale
            .as_ref()
            .map(|s| {
                s.state == ControlState::Done || s.state == ControlState::Requested
            })
            .unwrap_or(true)
}
