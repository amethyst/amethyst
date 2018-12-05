//! Util Resources

use amethyst_core::{
    specs::prelude::{DispatcherBuilder, Read, System, Write},
    timing::{duration_to_nanos, Time},
    SystemBundle,
};
use amethyst_error::Error;

use crate::circular_buffer::CircularBuffer;

/// The FPSCounter resource needed by the FPSCounterSystem.
///
/// Add it to your resources to be able to use the FPSCounterSystem.
///
/// ## Usage:
/// Get the FPSCounter resource from the world then call either `frame_fps` or `sampled_fps` to
/// get the FPS.
pub struct FPSCounter {
    buf: CircularBuffer<u64>,
    sum: u64,
}

impl Default for FPSCounter {
    fn default() -> Self {
        FPSCounter::new(20)
    }
}

impl FPSCounter {
    ///Creates a new FPSCounter that calculates the average fps over samplesize values.
    pub fn new(samplesize: usize) -> FPSCounter {
        FPSCounter {
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
/// to the [FPSCounter](../resources/struct.FPSCounter.html) resource with id 0
pub struct FPSCounterSystem;

impl<'a> System<'a> for FPSCounterSystem {
    type SystemData = (Read<'a, Time>, Write<'a, FPSCounter>);
    fn run(&mut self, (time, mut counter): Self::SystemData) {
        counter.push(duration_to_nanos(time.delta_real_time()));
        //Enable this to debug performance engine wide.
        //println!("Cur FPS: {}, Sampled: {}",counter.frame_fps(),counter.sampled_fps());
    }
}

///Automatically adds a FPSCounterSystem and a FPSCounter resource with the specified sample size.
#[derive(Default)]
pub struct FPSCounterBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for FPSCounterBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(FPSCounterSystem, "fps_counter_system", &[]);
        Ok(())
    }
}
