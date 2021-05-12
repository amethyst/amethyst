//! Allows you to automatically delete an entity after a set time has elapsed.

use amethyst_core::{ecs::*, Time};
use serde::{Deserialize, Serialize};
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// Destroys the entity to which this is attached at the specified time (in seconds).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestroyAtTime {
    /// The time at which the entity should be destroyed in seconds.
    /// Compared to `Time::absolute_time_seconds`.
    pub time: f64,
}

/// Destroys the entity to which this is attached after the specified time interval (in seconds).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestroyInTime {
    /// The amount of time before the entity should be destroyed in seconds.
    /// Compared to `Time::absolute_time_seconds`.
    pub timer: f64,
}

/// The system in charge of destroying entities with the `DestroyAtTime` component.
pub fn build_destroy_at_time_system() -> impl Runnable {
    SystemBuilder::new("destroy_at_time_system")
        .read_resource::<Time>()
        .with_query(<(Entity, Read<DestroyAtTime>)>::query())
        .build(move |commands, subworld, time, dat_query| {
            #[cfg(feature = "profiler")]
            profile_scope!("destroy_at_time_system");

            for (ent, dat) in dat_query.iter_mut(subworld) {
                if time.absolute_time().as_secs_f64() > dat.time {
                    commands.remove(*ent);
                }
            }
        })
}

/// The system in charge of destroying entities with the `DestroyInTime` component.
pub fn build_destroy_in_time_system() -> impl Runnable {
    SystemBuilder::new("destroy_in_time_system")
        .read_resource::<Time>()
        .with_query(<(Entity, Write<DestroyInTime>)>::query())
        .build(move |commands, subworld, time, dit_query| {
            #[cfg(feature = "profiler")]
            profile_scope!("destroy_in_time_system");

            for (ent, mut dit) in dit_query.iter_mut(subworld) {
                if dit.timer <= 0f64 {
                    commands.remove(*ent);
                }

                dit.timer -= time.delta_time().as_secs_f64();
            }
        })
}
