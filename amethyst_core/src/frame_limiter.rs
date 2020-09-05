//! Frame rate limiting.
//!
//! An amethyst [`Application`] runs in a loop, executing game update logic each frame. In
//! order to reduce CPU usage and keep frame timing predictable, amethyst uses a configurable
//! frame limiting strategy to introduce a delay before starting each frame if the previous
//! frame completed sufficiently quickly.
//!
//! The frame rate limiting strategy has two parts: A maximum frame rate, given as a number of
//! frames per second, and a strategy for returning any remaining time in the frame to the
//! operating system. Based on the specified maximum frame rate, each frame has a budget for
//! how long it can take. For example, at 60 fps each frame has 16.6 milliseconds to perform
//! any work it needs to. If a frame takes less time than is budgeted, amethyst will attempt to
//! yield the remaining time back to the operating system, using the chosen strategy.
//!
//! By default, amethyst will set the maximum frame rate to 144 fps, and will use a yield-only
//! limiting strategy.
//!
//! # Examples
//!
//! ```
//! use std::time::Duration;
//!
//! use amethyst::prelude::*;
//! use amethyst::core::frame_limiter::FrameRateLimitStrategy;
//!
//! # struct GameState;
//! # impl SimpleState for GameState {}
//! # fn main() -> amethyst::Result<()> {
//! let assets_dir = "./";
//! let mut game = Application::build(assets_dir, GameState)?
//!     .with_frame_limit(
//!         FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
//!         144,
//!     )
//!     .build(GameDataBuilder::new())?;
//! # Ok(())
//! # }
//! ```
//!
//! # Frame Rate Limiting Strategies
//!
//! The four possible strategies described by [`FrameRateLimitStrategy`] are as follows:
//!
//! * `Unlimited` will not try to limit the frame rate to the specified maximum. Amethyst
//!   will call [`thread::yield_now`] once and then continue to the next frame.
//! * `Yield` will call [`thread::yield_now`] repeatedly until the frame duration has
//!   passed. This will result in the most accurate frame timings, but effectively guarantees
//!   that one CPU core will be fully utilized during the frame's idle time.
//! * `Sleep` will attempt to sleep for the first half of the desired frame duration, and will then
//!   yield until the next frame starts. This approach, in contrast to `Yield`, helps reduce CPU usage
//!   while the game is idle. It yields for the remainder of the frame to reduce risk of fluctuations
//!   in frame timing caused by the imprecise nature of sleeps. This approach attempts to get the
//!   consistent frame timings of yielding, while reducing CPU usage compared to the yield-only
//!   approach.
//! * `SleepAndYield` differs from `Sleep` by letting you specify when to stop sleeping and start yielding,
//!   granting you complete control over the frame timings, whereas `Sleep` will sleep for half the frame and yield
//!   for the remainder of the frame.
//!
//! By default amethyst will use the `Yield` strategy, which is fine for desktop and console
//! games that aren't as affected by extra CPU usage. For mobile devices, the `Sleep` strategy
//! will help conserve battery life.
//!
//! `SleepAndYield` can potentially be as accurate as `Yield` while using less CPU time, but you
//! will have to test different grace period timings to determine how much time needs to be left
//! to ensure that the main thread doesn't sleep too long and miss the start of the next frame.
//!
//! [`Application`]: ../../amethyst/type.Application.html
//! [`FrameRateLimitStrategy`]: ./enum.FrameRateLimitStrategy.html
//! [`thread::yield_now`]: https://doc.rust-lang.org/std/thread/fn.yield_now.html
//! [`thread::sleep`]: https://doc.rust-lang.org/stable/std/thread/fn.sleep.html

use std::{
    assert,
    thread::{sleep, yield_now},
    time::{Duration, Instant},
};

use derive_new::new;
use serde::{Deserialize, Serialize};

/// Frame rate limiting strategy.
///
/// See the [module documentation] on the difference between sleeping and yielding, and when
/// these different strategies should be used.
///
/// [module documentation]: ./index.html#frame-rate-limiting-strategies
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FrameRateLimitStrategy {
    /// No limit, will do a single yield and then continue with the next frame.
    Unlimited,

    /// Yield repeatedly until the frame duration has passed.
    Yield,

    /// Use sleep and yield combined automatically based on target frame rate.
    ///
    /// Will sleep repeatedly until half the frame duration has passed, and will then yield
    /// repeatedly for the remaining frame time to prevent over-sleeping.
    Sleep,

    /// Use sleep and yield combined with an explicit sleep barrier.
    ///
    /// Will sleep repeatedly until the given duration remains, and will then yield repeatedly
    /// for the remaining frame time.
    SleepAndYield(Duration),
}

impl Default for FrameRateLimitStrategy {
    fn default() -> Self {
        FrameRateLimitStrategy::Yield
    }
}

/// Frame limiting configuration loaded from a configuration file.
///
/// Provides the configuration for a [`FrameLimiter`] using a configuration file. The config
/// file can be loaded using the methods of the [`Config`] trait.
///
/// # Examples
///
/// ```no_run
/// use amethyst::prelude::*;
/// use amethyst::core::frame_limiter::FrameRateLimitConfig;
///
/// let config = FrameRateLimitConfig::load("./config/frame_limiter.ron");
/// ```
///
/// [`FrameLimiter`]: ./struct.FrameLimiter.html
/// [`Config`]: ../../amethyst_config/trait.Config.html
#[derive(Debug, Clone, Deserialize, Serialize, new)]
pub struct FrameRateLimitConfig {
    /// Frame rate limiting strategy.
    pub strategy: FrameRateLimitStrategy,
    /// The FPS to limit the game loop execution.
    pub fps: u32,
}

impl Default for FrameRateLimitConfig {
    fn default() -> Self {
        FrameRateLimitConfig {
            fps: 144,
            strategy: Default::default(),
        }
    }
}

/// Frame limiter resource.
///
/// `FrameLimiter` is used internally by amethyst to limit the frame rate to the
/// rate specified by the user. It is added as a resource to the world so that user code may
/// change the frame rate limit at runtime if necessary.
#[derive(Debug)]
pub struct FrameLimiter {
    frame_duration: Duration,
    sleep_barrier: Duration,
    strategy: FrameRateLimitStrategy,
    last_call: Instant,
}

impl Default for FrameLimiter {
    fn default() -> Self {
        FrameLimiter::from_config(Default::default())
    }
}

impl FrameLimiter {
    /// Creates a new frame limiter.
    pub fn new(strategy: FrameRateLimitStrategy, fps: u32) -> Self {
        let mut s = Self {
            frame_duration: Duration::from_secs(0),
            sleep_barrier: Duration::from_secs(0),
            strategy: Default::default(),
            last_call: Instant::now(),
        };
        s.set_rate(strategy, fps);
        s
    }

    /// Sets the maximum fps and frame rate limiting strategy.
    pub fn set_rate(&mut self, strategy: FrameRateLimitStrategy, fps: u32) {
        assert!(fps > 0, "FrameLimiter::set_rate parameter `fps` is {}. This parameter must be greater than zero!");
        self.strategy = strategy;
        self.frame_duration = Duration::from_secs(1) / fps;
        self.sleep_barrier = self.frame_duration / 2;
    }

    /// Gets the assigned frame duration.
    pub fn get_frame_duration(&self) -> Duration {
        self.frame_duration
    }

    /// Creates a new frame limiter with the given config.
    pub fn from_config(config: FrameRateLimitConfig) -> Self {
        Self::new(config.strategy, config.fps)
    }

    /// Resets the frame start time to the current instant.
    ///
    /// This resets the frame limiter's internal tracking of when the last frame started to the
    /// current instant. Be careful when calling `start`, as doing so will cause the current
    /// frame to be longer than normal if not called at the very beginning of the frame.
    pub fn start(&mut self) {
        self.last_call = Instant::now();
    }

    /// Blocks the current thread until the allotted frame time has passed.
    ///
    /// `wait` is used internally by [`Application`] to limit the frame rate of the game
    /// to the configured rate. This should likely never be called directly by game logic.
    ///
    /// [`Application`]: ../../amethyst/type.Application.html
    pub fn wait(&mut self) {
        use self::FrameRateLimitStrategy::*;
        match self.strategy {
            Unlimited => yield_now(),

            Yield => self.do_yield(),

            Sleep => {
                self.do_sleep(self.sleep_barrier);
                self.do_yield();
            }

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
        loop {
            let elapsed = Instant::now() - self.last_call;
            if elapsed >= frame_duration {
                break;
            } else {
                sleep(frame_duration - elapsed);
            }
        }
    }
}
