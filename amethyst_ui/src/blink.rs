//! Module for the Blink component and BlinkSystem.

use amethyst_core::{
    ecs::{Component, DenseVecStorage, Entities, Join, Read, System, WriteStorage},
    HiddenComponent, Time,
};

/// # Blink Component
/// Periodically adds and removes a `HiddenComponent` Component on the entity this is attached to.
///
/// ## Visibility Period
/// During the first half period, the entity is visible.
/// [0, delay/2[
///
/// During the second half period, the entity is invisible.
/// [delay/2, delay]
pub struct BlinkComponent {
    /// Period of a full blink cycle.
    pub delay: f32,
    /// Timer value keeping track of the time during the blink cycle.
    pub timer: f32,
    /// Whether to use the scaled or unscaled time.
    pub absolute_time: bool,
}

impl Component for BlinkComponent {
    type Storage = DenseVecStorage<Self>;
}

/// System updating the `Blink` component.
pub struct BlinkSystem;

impl<'a> System<'a> for BlinkSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, HiddenComponent>,
        WriteStorage<'a, BlinkComponent>,
        Read<'a, Time>,
    );

    fn run(&mut self, (entities, mut hiddens, mut blinks, time): Self::SystemData) {
        let abs_sec = time.delta_seconds();
        let abs_unscaled_sec = time.delta_real_seconds();

        for (entity, blink) in (&*entities, &mut blinks).join() {
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

            match (on, hiddens.contains(entity)) {
                (true, false) => hiddens.insert(entity, HiddenComponent).unwrap_or_else(|_| {
                    panic!(
                        "Failed to insert HiddenComponent component for {:?}",
                        entity
                    )
                }),
                (false, true) => hiddens.remove(entity),
                _ => None,
            };
        }
    }
}
