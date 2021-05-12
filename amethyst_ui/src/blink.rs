//! Module for the Blink component and BlinkSystem.

use amethyst_core::{ecs::*, Hidden, Time};
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// # Blink Component
/// Periodically adds and removes a `Hidden` Component on the entity this is attached to.
///
/// ## Visibility Period
/// During the first half period, the entity is visible.
/// [0, delay/2[
///
/// During the second half period, the entity is invisible.
/// [delay/2, delay]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Blink {
    /// Period of a full blink cycle.
    pub delay: f32,
    /// Timer value keeping track of the time during the blink cycle.
    pub timer: f32,
    /// Whether to use the scaled or unscaled time.
    pub absolute_time: bool,
}

/// System updating the `Blink` component.
#[derive(Debug)]
pub struct BlinkSystem;

impl System for BlinkSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("BlinkSystem")
                .read_resource::<Time>()
                .with_query(<&mut Hidden>::query())
                .with_query(<(Entity, Write<Blink>)>::query())
                .build(move |commands, world, time, (hiddens, blinks)| {
                    #[cfg(feature = "profiler")]
                    profile_scope!("blink_system");

                    let abs_sec = time.delta_time().as_secs_f32();
                    let abs_unscaled_sec = time.delta_real_time().as_secs_f32();

                    let (mut blinks_world, mut subworld) = world.split_for_query(&blinks);

                    blinks.for_each_mut(&mut blinks_world, |(entity, mut blink)| {
                        if blink.absolute_time {
                            blink.timer += abs_unscaled_sec;
                        } else {
                            blink.timer += abs_sec;
                        }

                        // Reset timer because we ended the last cycle.
                        // Keeps the overflow time.
                        if blink.timer > blink.delay {
                            blink.timer -= blink.delay;
                        }

                        // We could cache the division, but that would require a stricter api on Blink.
                        let on = blink.timer < blink.delay / 2.0;

                        match (on, hiddens.get_mut(&mut subworld, *entity).is_ok()) {
                            (true, false) => {
                                commands.add_component(*entity, Hidden);
                            }
                            (false, true) => {
                                commands.remove_component::<Hidden>(*entity);
                            }
                            _ => {}
                        };
                    });
                }),
        )
    }
}
