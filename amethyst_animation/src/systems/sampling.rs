use std::{marker, time::Duration};

use itertools::Itertools;
use minterpolate::InterpolationPrimitive;

use amethyst_assets::AssetStorage;
use amethyst_core::{
    duration_to_nanos, duration_to_secs,
    ecs::prelude::{Component, Join, Read, System, WriteStorage},
    nanos_to_duration, secs_to_duration, Time,
};

use crate::resources::{
    AnimationSampling, ApplyData, BlendMethod, ControlState, EndControl, Sampler, SamplerControl,
    SamplerControlSet,
};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

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
#[derive(Default, Debug)]
pub struct SamplerInterpolationSystem<T>
where
    T: AnimationSampling,
{
    m: marker::PhantomData<T>,
    inner: Vec<(f32, T::Channel, T::Primitive)>,
    channels: Vec<T::Channel>,
}

impl<T> SamplerInterpolationSystem<T>
where
    T: AnimationSampling,
{
    /// Creates a new `SamplerInterpolationSystem`
    pub fn new() -> Self {
        Self {
            m: marker::PhantomData,
            inner: Vec::default(),
            channels: Vec::default(),
        }
    }
}

impl<'a, T> System<'a> for SamplerInterpolationSystem<T>
where
    T: AnimationSampling + Component,
{
    type SystemData = (
        Read<'a, Time>,
        Read<'a, AssetStorage<Sampler<T::Primitive>>>,
        WriteStorage<'a, SamplerControlSet<T>>,
        WriteStorage<'a, T>,
        <T as ApplyData<'a>>::ApplyData,
    );

    fn run(&mut self, (time, samplers, mut control_sets, mut comps, apply_data): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("sampler_interpolation_system");

        for (control_set, comp) in (&mut control_sets, &mut comps).join() {
            self.inner.clear();
            for control in control_set.samplers.iter_mut() {
                if let Some(ref sampler) = samplers.get(&control.sampler) {
                    process_sampler(control, sampler, &time, &mut self.inner);
                }
            }
            if !self.inner.is_empty() {
                self.channels.clear();
                self.channels
                    .extend(self.inner.iter().map(|o| &o.1).unique().cloned());
                for channel in &self.channels {
                    match comp.blend_method(channel) {
                        None => {
                            if let Some(p) = self
                                .inner
                                .iter()
                                .filter(|p| p.1 == *channel)
                                .map(|p| p.2.clone())
                                .last()
                            {
                                comp.apply_sample(channel, &p, &apply_data);
                            }
                        }

                        Some(BlendMethod::Linear) => {
                            if let Some(p) = linear_blend::<T>(channel, &self.inner) {
                                comp.apply_sample(channel, &p, &apply_data);
                            }
                        }
                    }
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
                    duration_to_secs(duration),
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
                duration + secs_to_duration(time.delta_seconds() * control.rate_multiplier);
            let last_frame = sampler
                .input
                .last()
                .cloned()
                .map(secs_to_duration)
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
    let animation_duration = duration_to_nanos(last_frame);
    let current_duration = duration_to_nanos(duration);
    let remain_duration = current_duration % animation_duration;
    let loops = current_duration / animation_duration;
    (nanos_to_duration(remain_duration), loops as u32)
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
