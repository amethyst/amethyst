use std::{collections::HashSet, marker::PhantomData, time::Duration};

use amethyst_assets::AssetStorage;
use amethyst_core::{
    ecs::{systems::ParallelRunnable, *},
    Time,
};
use derivative::Derivative;
use log::debug;
use minterpolate::InterpolationPrimitive;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::resources::{
    AnimationSampling, BlendMethod, ControlState, EndControl, Sampler, SamplerControl,
    SamplerControlSet,
};

/// System for interpolating active samplers.
///
/// If other forms of animation is needed, this can be used in isolation, have no direct dependency
/// on `AnimationControlSystem`.
///
/// Will process all active `SamplerControlSet`, and update the target component for the entity they
/// belong to.
///
/// ### Type parameters:
///
/// - `T`: the component type that the animation should be applied to
#[derive(Derivative)]
#[derivative(Default)]
pub(crate) struct SamplerInterpolationSystem<T: AnimationSampling> {
    _marker: PhantomData<T>,
}

impl<T> System for SamplerInterpolationSystem<T>
where
    T: AnimationSampling + std::fmt::Debug,
{
    fn build(self) -> Box<dyn ParallelRunnable> {
        let mut inner = Vec::default();
        let mut channels = Vec::default();

        Box::new(
            SystemBuilder::new("SamplerInterpolationSystem")
                .read_resource::<Time>()
                .read_resource::<AssetStorage<Sampler<T::Primitive>>>()
                .with_query(<(Write<SamplerControlSet<T>>, Write<T>)>::query())
                .build(move |commands, world, (time, samplers), query| {
                    #[cfg(feature = "profiler")]
                    profile_scope!("sampler_interpolation_system");

                    for (control_set, comp) in query.iter_mut(world) {
                        debug!("Processing SamplerControlSet: {:?}", control_set);

                        inner.clear();

                        for control in control_set.samplers.iter_mut() {
                            if let Some(ref sampler) = samplers.get(&control.sampler) {
                                process_sampler(control, sampler, &time, &mut inner);
                            }
                        }
                        if !inner.is_empty() {
                            channels.clear();
                            channels
                                .extend(inner.iter().map(|o| o.1.clone()).collect::<HashSet<_>>());
                            for channel in &channels {
                                match comp.blend_method(channel) {
                                    None => {
                                        if let Some(p) = inner
                                            .iter()
                                            .filter(|p| p.1 == *channel)
                                            .map(|p| p.2.clone())
                                            .last()
                                        {
                                            comp.apply_sample(channel, &p, commands);
                                        }
                                    }

                                    Some(BlendMethod::Linear) => {
                                        if let Some(p) = linear_blend::<T>(channel, &inner) {
                                            comp.apply_sample(channel, &p, commands);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }),
        )
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
    time: &Time,
    output: &mut Vec<(f32, T::Channel, T::Primitive)>,
) where
    T: AnimationSampling,
{
    use crate::resources::ControlState::*;

    let (new_state, new_end) = update_duration_and_check(&control, sampler, time);

    // If a new end condition has been computed, update in control state
    if let Some(end) = new_end {
        control.end = end;
    }

    // Do sampling
    match new_state {
        Running(duration) | Paused(duration) => {
            output.push((
                control.blend_weight,
                control.channel.clone(),
                sampler.function.interpolate(
                    duration.as_secs_f32(),
                    &sampler.input,
                    &sampler.output,
                    false,
                ),
            ));
        }
        Done => {
            if let EndControl::Normal = control.end {
                output.push((
                    control.blend_weight,
                    control.channel.clone(),
                    control.after.clone(),
                ));
            }
            if let EndControl::Stay = control.end {
                let last_frame = sampler.input.last().cloned().unwrap_or(0.);

                output.push((
                    control.blend_weight,
                    control.channel.clone(),
                    sampler.function.interpolate(
                        last_frame,
                        &sampler.input,
                        &sampler.output,
                        false,
                    ),
                ));
            }
        }
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
    use crate::resources::ControlState::*;
    // Update state with new duration
    // Check duration for end of sampling
    match control.state {
        // requested sampling => start interpolating
        Requested => (Running(Duration::from_secs(0)), None),

        // deferred start that should start now
        Deferred(dur) => (Running(dur), None),

        // abort sampling => end interpolating
        Abort => (Done, None),

        // sampling is running, update duration and check end condition
        Running(duration) => {
            let current_dur =
                duration + Duration::from_secs_f32(time.delta_time().as_secs_f32() * control.rate_multiplier);
            let last_frame = sampler
                .input
                .last()
                .cloned()
                .map(|t| Duration::from_secs_f32(t))
                .unwrap_or_else(|| Duration::from_secs(0));
            // duration is past last frame of sampling
            if current_dur > last_frame {
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
    let animation_duration = last_frame.as_nanos();
    let current_duration = duration.as_nanos();
    let remain_duration = current_duration % animation_duration;
    let loops = current_duration / animation_duration;
    (Duration::from_nanos(remain_duration as u64), loops as u32)
}

fn linear_blend<T>(
    channel: &T::Channel,
    output: &[(f32, T::Channel, T::Primitive)],
) -> Option<T::Primitive>
where
    T: AnimationSampling,
{
    let total_blend_weight = output.iter().filter(|o| o.1 == *channel).map(|o| o.0).sum();
    if total_blend_weight == 0. {
        None
    } else {
        Some(
            output
                .iter()
                .filter(|o| o.1 == *channel)
                .map(|o| single_blend::<T>(total_blend_weight, o))
                .fold(T::default_primitive(channel), |acc, p| acc.add(&p)),
        )
    }
}

fn single_blend<T>(
    total: f32,
    &(ref weight, _, ref primitive): &(f32, T::Channel, T::Primitive),
) -> T::Primitive
where
    T: AnimationSampling,
{
    primitive.mul(*weight / total)
}
