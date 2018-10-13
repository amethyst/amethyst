//! Allows you to automatically delete an entity after a set time has elapsed.

use amethyst_core::specs::{
    Component, DenseVecStorage, Entities, Join, Read, ReadStorage, System, WriteStorage,
};
use amethyst_core::timing::Time;

/// Destroys the entity to which this is attached at the specified time (in seconds).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestroyAtTime {
    /// The time at which the entity should be destroyed in seconds.
    /// Compared to `Time::absolute_time_seconds`.
    pub time: f64,
}

impl Component for DestroyAtTime {
    type Storage = DenseVecStorage<Self>;
}

/// Destroys the entity to which this is attached after the specified time interval (in seconds).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestroyInTime {
    /// The amount of time before the entity should be destroyed in seconds.
    /// Compared to `Time::absolute_time_seconds`.
    pub timer: f64,
}

impl Component for DestroyInTime {
    type Storage = DenseVecStorage<Self>;
}

/// The system in charge of destroying entities with the `DestroyAtTime` or `DestroyInTime` components.
pub struct TimedDestroySystem;

impl<'a> System<'a> for TimedDestroySystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, DestroyAtTime>,
        WriteStorage<'a, DestroyInTime>,
        Read<'a, Time>,
    );
    fn run(&mut self, (entities, dat, mut dit, time): Self::SystemData) {
        for (e, d) in (&*entities, &dat).join() {
            if time.absolute_time_seconds() > d.time {
                if let Err(err) = entities.delete(e) {
                    error!("Failed to delete entity: {:?}", err);
                }
            }
        }

        for (e, mut d) in (&*entities, &mut dit).join() {
            if d.timer <= 0.0 {
                if let Err(err) = entities.delete(e) {
                    error!("Failed to delete entity: {:?}", err);
                }
            }
            d.timer -= time.delta_seconds() as f64;
        }
    }
}
