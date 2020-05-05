//! Util Resources

use amethyst_core::{
    ecs::prelude::{DispatcherBuilder, Read, System, World, Write},
    timing::{duration_to_nanos, Time},
    SystemBundle,
};
use amethyst_error::Error;

use crate::circular_buffer::CircularBuffer;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// The FpsCounter resource needed by the FpsCounterSystem.
///
/// Add it to your resources to be able to use the FpsCounterSystem.
///
/// # Usage
/// Get the FpsCounter resource from the world then call either `frame_fps` or `sampled_fps` to
/// get the FPS.
///
/// frame_fps will return the framerate of the current frame. That is, the framerate at which the
/// game would be running if all frames were exactly like this one.
/// sampled_fps will return the averaged framerate. This gives a better approximation of the "felt"
/// framerate by the user.
///
/// # Example
/// ```rust
/// # use amethyst_utils::fps_counter::FpsCounter;
/// # use amethyst_core::ecs::{World, WorldExt};
/// # let mut world = World::new();
/// # let counter = FpsCounter::new(2);
/// # world.insert(counter);
/// let mut counter = world.write_resource::<FpsCounter>();
///
/// ```
#[derive(Debug)]
pub struct FpsCounter {
    buf: CircularBuffer<u64>,
    sum: u64,
}

impl Default for FpsCounter {
    fn default() -> Self {
        FpsCounter::new(20)
    }
}

impl FpsCounter {
    ///Creates a new FpsCounter that calculates the average fps over samplesize values.
    pub fn new(samplesize: usize) -> FpsCounter {
        FpsCounter {
            buf: CircularBuffer::<u64>::new(samplesize),
            sum: 0,
        }
    }

    ///Add a new delta time value.
    pub fn push(&mut self, elem: u64) {
        self.sum += elem;
        if let Some(front) = self.buf.push(elem) {
            self.sum -= front;
        }
    }

    ///Get the fps of the this frame.
    pub fn frame_fps(&self) -> f32 {
        if let Some(back) = self.buf.queue().back() {
            return 1.0e9 / *back as f32;
        }
        0.0
    }

    ///Get the average fps over the samplesize frames.
    pub fn sampled_fps(&self) -> f32 {
        if self.sum == 0 || self.buf.queue().is_empty() {
            return 0.0;
        }
        1.0e9 * self.buf.queue().len() as f32 / self.sum as f32
    }
}

/// Add this system to your game to automatically push FPS values
/// to the [FpsCounter](../resources/struct.FpsCounter.html) resource with id 0
#[derive(Debug)]
pub struct FpsCounterSystem;

impl<'a> System<'a> for FpsCounterSystem {
    type SystemData = (Read<'a, Time>, Write<'a, FpsCounter>);
    fn run(&mut self, (time, mut counter): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("fps_counter_system");

        counter.push(duration_to_nanos(time.delta_real_time()));
        //Enable this to debug performance engine wide.
        log::debug!(
            "Cur FPS: {}, Sampled: {}",
            counter.frame_fps(),
            counter.sampled_fps()
        );
    }
}

///Automatically adds a FpsCounterSystem and a FpsCounter resource with the specified sample size.
#[derive(Default, Debug)]
pub struct FpsCounterBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for FpsCounterBundle {
    fn build(
        self,
        _world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add(FpsCounterSystem, "fps_counter_system", &[]);
        Ok(())
    }
}
