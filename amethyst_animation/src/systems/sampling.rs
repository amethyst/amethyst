use std::time::Duration;

use amethyst_assets::AssetStorage;
use amethyst_core::{duration_to_nanos, duration_to_secs, nanos_to_duration, secs_to_duration,
                    LocalTransform, Time};
use specs::{Fetch, Join, System, WriteStorage};

use interpolation::Interpolate;
use resources::{ControlState, EndControl, RestState, Sampler, SamplerControl, SamplerControlSet};

/// System for interpolating active samplers.
///
/// If other forms of animation is needed, this can be used in isolation, have no direct dependency
/// on `AnimationControlSystem`.
///
/// Will process all active `SamplerControlSet`, and update the `LocalTransform` for the entity they
/// belong to.
#[derive(Default)]
pub struct SamplerInterpolationSystem;

impl SamplerInterpolationSystem {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a> System<'a> for SamplerInterpolationSystem {
    type SystemData = (
        Fetch<'a, Time>,
        Fetch<'a, AssetStorage<Sampler>>,
        WriteStorage<'a, SamplerControlSet>,
        WriteStorage<'a, LocalTransform>,
    );

    fn run(&mut self, (time, samplers, mut controls, mut transforms): Self::SystemData) {
        for (control_set, transform) in (&mut controls, &mut transforms).join() {
            if let Some((ref mut control, sampler)) = control_set
                .translation
                .as_mut()
                .and_then(|c| samplers.get(&c.sampler).and_then(|s| Some((c, s))))
            {
                process_sampler(control, sampler, transform, &time);
            }
            if let Some((ref mut control, sampler)) = control_set
                .rotation
                .as_mut()
                .and_then(|c| samplers.get(&c.sampler).and_then(|s| Some((c, s))))
            {
                process_sampler(control, sampler, transform, &time);
            }
            if let Some((ref mut control, sampler)) = control_set
                .scale
                .as_mut()
                .and_then(|c| samplers.get(&c.sampler).and_then(|s| Some((c, s))))
            {
                process_sampler(control, sampler, transform, &time);
            }
        }
    }
}

/// Process a single `SamplerControl` object.
///
/// ## Parameters:
///
/// - `control`: sampler control object
/// - `sampler`: the sampler reference from the control object
/// - `transform`: the `LocalTransform` to update
/// - `now`: synchronized `Instant` for the current frame
fn process_sampler(
    control: &mut SamplerControl,
    sampler: &Sampler,
    transform: &mut LocalTransform,
    time: &Time,
) {
    use resources::ControlState::*;

    let (new_state, new_end) = update_duration_and_check(&control, sampler, time);

    // If a new end condition has been computed, update in control state
    if let Some(end) = new_end {
        control.end = end.clone();
    }

    // Do sampling
    match new_state {
        Running(duration) | Paused(duration) => {
            do_sampling(&sampler, &duration, transform);
        }
        Done => do_end_control(&control, transform),
        _ => {}
    }

    // Update state for next iteration
    control.state = new_state;
}

/// Called on samplers that have finished.
fn do_end_control(control: &SamplerControl, transform: &mut LocalTransform) {
    match control.end {
        EndControl::Normal => match control.after {
            RestState::Translation(tr) => transform.translation = tr.into(),
            RestState::Rotation(r) => transform.rotation = r.into(),
            RestState::Scale(s) => transform.scale = s.into(),
        },
        // looping is handled during duration update
        _ => {}
    }
}

/// Update durations, check if the sampler is finished, start new samplers, and check for aborted
/// samplers.
///
/// ## Parameters
///
/// - `control`: sampler control object
/// - `sampler`: sampler reference from control
/// - `now`: synchronized `Instant` for the current frame
///
/// ## Returns
///
/// Will return the new state of the sampling, and optionally a new end control state (for looping)
fn update_duration_and_check(
    control: &SamplerControl,
    sampler: &Sampler,
    time: &Time,
) -> (ControlState, Option<EndControl>) {
    use resources::ControlState::*;
    // Update state with new duration
    // Check duration for end of sampling
    match control.state {
        // requested sampling => start interpolating
        Requested => (Running(Duration::from_secs(0)), None),

        // abort sampling => end interpolating
        Abort => (Done, None),

        // sampling is running, update duration and check end condition
        Running(duration) => {
            let zero = Duration::from_secs(0);
            let current_dur = duration + time.delta_time();
            let last_frame = sampler
                .input
                .last()
                .cloned()
                .map(secs_to_duration)
                .unwrap_or(zero.clone());
            // duration is past last frame of sampling
            if last_frame != zero && current_dur > last_frame {
                // Check end conditions
                match control.end {
                    // Do loop control
                    EndControl::Loop(Some(i)) if i <= 1 => (Done, Some(EndControl::Normal)),
                    EndControl::Loop(None) => {
                        (Running(next_duration(last_frame, current_dur).0), None)
                    }
                    EndControl::Loop(Some(i)) => {
                        let (next_dur, loops_removed) = next_duration(last_frame, current_dur);
                        let remaining_loops = i - loops_removed;
                        if remaining_loops <= 1 {
                            (Done, Some(EndControl::Normal))
                        } else {
                            (
                                Running(next_dur),
                                Some(EndControl::Loop(Some(remaining_loops))),
                            )
                        }
                    }
                    // All other end cases will be handled during sampling
                    _ => (Done, None),
                }
            } else {
                // last frame not reached, keep sampling
                (Running(current_dur), None)
            }
        }

        // Done and paused will be handled during sampling
        ref state => (state.clone(), None),
    }
}

fn next_duration(last_frame: Duration, duration: Duration) -> (Duration, u32) {
    let animation_duration = duration_to_nanos(last_frame);
    let current_duration = duration_to_nanos(duration);
    let remain_duration = current_duration % animation_duration;
    let loops = current_duration / animation_duration;
    (nanos_to_duration(remain_duration), loops as u32)
}

fn do_sampling(sampler: &Sampler, duration: &Duration, local_transform: &mut LocalTransform) {
    use resources::AnimationOutput::*;
    let dur_s = duration_to_secs(*duration);
    match sampler.output {
        Translation(ref ts) => {
            local_transform.translation = sampler
                .ty
                .interpolate(dur_s, &sampler.input, ts, false)
                .into()
        }
        Rotation(ref rs) => {
            local_transform.rotation = sampler
                .ty
                .interpolate(dur_s, &sampler.input, rs, true)
                .into()
        }
        Scale(ref ss) => {
            local_transform.scale = sampler
                .ty
                .interpolate(dur_s, &sampler.input, ss, false)
                .into()
        }
    }
}
