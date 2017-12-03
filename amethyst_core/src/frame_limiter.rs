//! Frame limiter

use std::thread::{sleep, yield_now};
use std::time::{Duration, Instant};

/// Frame rate limiting strategy
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FrameRateLimitStrategy {
    /// No limit, will do a single yield, and then continue with the next frame
    Unlimited,

    /// Use yield until the full frame duration has passed
    Yield,

    /// Use sleep until the full frame duration has passed
    Sleep,

    /// Use sleep and yield combined, will use sleep strategy until the given duration remains,
    /// and then swap to yield strategy.
    SleepAndYield(Duration),
}

impl Default for FrameRateLimitStrategy {
    fn default() -> Self {
        FrameRateLimitStrategy::Yield
    }
}

/// Frame limit config
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FrameRateLimitConfig {
    strategy: FrameRateLimitStrategy,
    fps: u32,
}

impl Default for FrameRateLimitConfig {
    fn default() -> Self {
        FrameRateLimitConfig {
            fps: 144,
            strategy: Default::default(),
        }
    }
}

/// Frame limiter
#[derive(Debug)]
pub struct FrameLimiter {
    frame_duration: Duration,
    strategy: FrameRateLimitStrategy,
    last_call: Instant,
    zero: Duration,
}

impl Default for FrameLimiter {
    fn default() -> Self {
        FrameLimiter::from_config(Default::default())
    }
}

impl FrameLimiter {
    /// Create a new frame limiter with the given config
    pub fn new(strategy: FrameRateLimitStrategy, fps: u32) -> Self {
        let mut s = Self {
            frame_duration: Duration::from_secs(0),
            strategy: Default::default(),
            last_call: Instant::now(),
            zero: Duration::from_millis(0),
        };
        s.set_rate(strategy, fps);
        s
    }

    /// Set the strategy and fps
    pub fn set_rate(&mut self, mut strategy: FrameRateLimitStrategy, mut fps: u32) {
        if fps == 0 {
            strategy = FrameRateLimitStrategy::Unlimited;
            fps = 144;
        }
        self.strategy = strategy;
        self.frame_duration = Duration::from_secs(1) / fps;
    }

    /// Create a new frame limiter with the given config
    pub fn from_config(config: FrameRateLimitConfig) -> Self {
        Self::new(config.strategy, config.fps)
    }

    /// Start the limiter
    pub fn start(&mut self) {
        self.last_call = Instant::now();
    }

    /// Wait until the frame has passed
    pub fn wait(&mut self) {
        use self::FrameRateLimitStrategy::*;
        match self.strategy {
            Unlimited => yield_now(),

            Yield => self.do_yield(),

            Sleep => self.do_sleep(self.zero),

            SleepAndYield(dur) => {
                self.do_sleep(dur);
                self.do_yield();
            }
        }
        self.last_call = Instant::now();
    }

    fn do_yield(&self) {
        while Instant::now() - self.last_call < self.frame_duration {
            yield_now();
        }
    }

    fn do_sleep(&self, stop_on_remaining: Duration) {
        let frame_duration = self.frame_duration - stop_on_remaining;
        while Instant::now() - self.last_call < frame_duration {
            sleep(self.zero)
        }
    }
}
