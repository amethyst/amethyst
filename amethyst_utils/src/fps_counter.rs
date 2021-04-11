//! Util Resources

use amethyst_core::{ecs::*, Time};
use amethyst_error::Error;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::circular_buffer::CircularBuffer;

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
/// ```
/// # use amethyst::utils::fps_counter::FpsCounter;
/// # use amethyst::core::ecs::{World, Resources};
/// # let mut world = World::default();
/// let mut resources = Resources::default();
/// let counter = FpsCounter::new(2);
/// resources.insert(counter);
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
struct FpsCounterSystem;

impl System for FpsCounterSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("fps_counter_system")
                .read_resource::<Time>()
                .write_resource::<FpsCounter>()
                .build(move |_, _, (time, counter), _| {
                    #[cfg(feature = "profiler")]
                    profile_scope!("fps_counter_system");

                    counter.push(time.delta_real_time().as_nanos() as u64);
                    //Enable this to debug performance engine wide.
                    log::debug!(
                        "Cur FPS: {}, Sampled: {}",
                        counter.frame_fps(),
                        counter.sampled_fps()
                    );
                }),
        )
    }
}

///Automatically adds a `FpsCounterSystem` and a [`FpsCounter`] resource with the specified sample size.
#[derive(Default, Debug)]
pub struct FpsCounterBundle {
    samplesize: Option<usize>,
}

impl FpsCounterBundle {
    /// Create a new [`FpsCounterBundle`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the sample size the [`FpsCounter`] uses. The default is 20.
    pub fn sample_size(self, samplesize: usize) -> Self {
        Self {
            samplesize: Some(samplesize),
        }
    }
}

impl SystemBundle for FpsCounterBundle {
    fn load(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        resources.insert(
            self.samplesize
                .map_or_else(FpsCounter::default, FpsCounter::new),
        );
        builder.add_system(FpsCounterSystem);
        Ok(())
    }
}
