//! Util Systems

use ecs::util::resources::FPSCounter;
use specs::{Fetch, FetchMut, System};
use timing::Time;
use util::time::duration_to_nanos;

///FPSCounterSystem
///Add this system to your game to calculate FPS
/// ## Examples
/// ~~~no_run
/// use ecs::util::resources::FPSCounter;
/// let counter = FPSCounter::new(10);
/// counter.push(duration_to_secs(&time.delta_time)); //time.delta_time is a Duration of the delta time of this frame
/// println!("Cur FPS: {}, Sampled: {}",counter.frame_fps(),counter.sampled_fps());
/// ~~~

pub struct FPSCounterSystem;

impl<'a> System<'a> for FPSCounterSystem {
    type SystemData = (Fetch<'a, Time>, FetchMut<'a, FPSCounter>);
    fn run(&mut self, (time, mut counter): Self::SystemData) {
        counter.push(duration_to_nanos(time.delta_time));
        //Enable this to debug performance engine wide.
        //println!("Cur FPS: {}, Sampled: {}",counter.frame_fps(),counter.sampled_fps());
    }
}
