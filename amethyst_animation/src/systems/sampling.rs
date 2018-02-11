use std::marker;
use std::time::Duration;

use amethyst_assets::AssetStorage;
use amethyst_core::{duration_to_nanos, duration_to_secs, nanos_to_duration, secs_to_duration, Time};
use specs::{Component, Fetch, Join, System, WriteStorage};

use resources::{AnimationSampling, ControlState, EndControl, Sampler, SamplerControl,
                SamplerControlSet};

/// System for interpolating active samplers.
///
/// If other forms of animation is needed, this can be used in isolation, have no direct dependency
/// on `AnimationControlSystem`.
///
/// Will process all active `SamplerControlSet`, and update the target component for the entity they
/// belong to.
#[derive(Default)]
pub struct SamplerInterpolationSystem<T> {
    m: marker::PhantomData<T>,
}

impl<T> SamplerInterpolationSystem<T> {
    pub fn new() -> Self {
        Self {
            m: marker::PhantomData,
        }
    }
}

impl<'a, T> System<'a> for SamplerInterpolationSystem<T>
where
    T: AnimationSampling + Component,
{
    type SystemData = (
        Fetch<'a, Time>,
        Fetch<'a, AssetStorage<Sampler<T::Primitive>>>,
        WriteStorage<'a, SamplerControlSet<T>>,
        WriteStorage<'a, T>,
    );

    fn run(&mut self, (time, samplers, mut control_sets, mut comps): Self::SystemData) {
        for (control_set, comp) in (&mut control_sets, &mut comps).join() {
            for control in control_set.samplers.values_mut() {
                if let Some(ref sampler) = samplers.get(&control.sampler) {
                    process_sampler(control, sampler, comp, &time);
                }
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
/// - `component`: the component to update
/// - `now`: synchronized `Instant` for the current frame
fn process_sampler<T>(
    control: &mut SamplerControl<T>,
    sampler: &Sampler<T::Primitive>,
    component: &mut T,
    time: &Time,
) where
    T: AnimationSampling,
{
    use resources::ControlState::*;

    let (new_state, new_end) = update_duration_and_check(&control, sampler, time);

    // If a new end condition has been computed, update in control state
    if let Some(end) = new_end {
        control.end = end;
    }

    // Do sampling
    match new_state {
        Running(duration) | Paused(duration) => {
            do_sampling(&sampler, &duration, &control.channel, component);
        }
        Done => do_end_control(&control, component),
        _ => {}
    }

    // Update state for next iteration
    control.state = new_state;
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
fn update_duration_and_check<T>(
    control: &SamplerControl<T>,
    sampler: &Sampler<T::Primitive>,
    time: &Time,
) -> (ControlState, Option<EndControl>)
where
    T: AnimationSampling,
{
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
            let current_dur =
                duration + secs_to_duration(time.delta_seconds() * control.rate_multiplier);
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

/// Called on samplers that have finished.
fn do_end_control<T>(control: &SamplerControl<T>, component: &mut T)
where
    T: AnimationSampling,
{
    if let EndControl::Normal = control.end {
        component.apply_sample(&control.channel, &control.after);
    }
}

fn do_sampling<T>(
    sampler: &Sampler<T::Primitive>,
    duration: &Duration,
    channel: &T::Channel,
    component: &mut T,
) where
    T: AnimationSampling,
{
    component.apply_sample(
        channel,
        &sampler.function.interpolate(
            duration_to_secs(*duration),
            &sampler.input,
            &sampler.output,
            false,
        ),
    );
}
