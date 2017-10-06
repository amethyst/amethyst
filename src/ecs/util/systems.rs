//! Util Systems

use ecs::util::resources::FPSCounter;
use specs::{Fetch, FetchMut, System};
use timing::Time;
use util::time::duration_to_nanos;

/// Add this system to your game to automatically push FPS values
/// to the [FPSCounter](FPSCounter.html) resource with id 0
pub struct FPSCounterSystem;

impl<'a> System<'a> for FPSCounterSystem {
    type SystemData = (Fetch<'a, Time>, FetchMut<'a, FPSCounter>);
    fn run(&mut self, (time, mut counter): Self::SystemData) {
        counter.push(duration_to_nanos(time.delta_time));
        //Enable this to debug performance engine wide.
        //println!("Cur FPS: {}, Sampled: {}",counter.frame_fps(),counter.sampled_fps());
    }
}
