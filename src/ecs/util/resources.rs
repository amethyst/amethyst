//! Util Resources

use util::circular_buffer::CircularBuffer;

/// The FPSCounter resource needed by the FPSCounterSystem.
///
/// Add it to your resources with id 0 to be able to use the FPSCounterSystem.
///
/// # Usage:
/// Get the FPSCounter resource from the world then call something like:
///
/// ```rust,no_run
/// println!("Cur FPS: {}, Sampled: {}",counter.frame_fps(),counter.sampled_fps());
/// ```
///
pub struct FPSCounter {
    buf: CircularBuffer<u64>,
    sum: u64,
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
        if self.sum == 0 || self.buf.queue().len() == 0 {
            return 0.0;
        }
        1.0e9 * self.buf.queue().len() as f32 / self.sum as f32
    }
}
