#![crate_name = "amethyst_context"]
#![crate_type = "lib"]
#![doc(html_logo_url = "http://tinyurl.com/hgsb45k")]

//! A way to manage engine resources.
//!
//! # Basic usage:

//! ```rust
//! extern crate amethyst_config;
//! extern crate amethyst_context;
//!
//! use amethyst_config::Element;
//! use amethyst_context::{Context, Config};
//!
//!
//! fn main() {
//!     let config = Config::default();
//!     let context = Context::new(config);
//!     // Now resources provided by Context are available
//! }
//! ```
//! See `amethyst/examples/window.rs` for an example.

#[macro_use]
extern crate amethyst_config;

use amethyst_config::Element;
use std::path::Path;

pub mod video_context;
pub mod broadcaster;
pub mod timing;
pub mod event;
pub mod renderer;
pub mod asset_manager;
use video_context::{VideoContext, DisplayConfig};
use renderer::Renderer;
use asset_manager::AssetManager;
use broadcaster::Broadcaster;
use event::EngineEvent;
use timing::{Duration, SteadyTime};

config!(
    /// Contains configs for resources provided by `Context`
    struct Config {
    pub display_config: DisplayConfig = DisplayConfig::default(),
});

/// Contains all engine resources which must be shared by multiple parties, in particular `Renderer` and `Broadcaster`.
/// An `Arc<Mutex<Context>>` is passed to every `Processor` run by the engine and a `&mut Context` is passed to every `State`
/// method.
pub struct Context {
    // pub video_context: VideoContext,
    pub renderer: Renderer,
    pub asset_manager: AssetManager,
    pub broadcaster: Broadcaster,
    pub delta_time: Duration,
    pub fixed_step: Duration,
    pub last_fixed_update: SteadyTime,
}

unsafe impl Send for Context {}

impl Context {
    /// Create a `Context` configured according to `Config`
    pub fn new(config: Config) -> Context {
        let (video_context, factory) = VideoContext::new(&config.display_config);
        let renderer = Renderer::new(video_context);
        let asset_manager = AssetManager::new(factory);
        let mut broadcaster = Broadcaster::new();
        broadcaster.register::<EngineEvent>();

        Context {
            renderer: renderer,
            asset_manager: asset_manager,
            broadcaster: broadcaster,
            delta_time: Duration::zero(),
            fixed_step: Duration::microseconds(16666),
            last_fixed_update: SteadyTime::now(),
        }
    }

    /// Return a vector containing all engine events
    /// that have occured since the last call of this method
    pub fn poll_engine_events(&mut self) -> Vec<EngineEvent> {
        let mut events = vec!();
        let video_context = self.renderer.mut_video_context();
        let ref mut window = video_context.window;

        for event in window.poll_events() {
            let event = EngineEvent::new(event);
            events.push(event);
        }
        events
    }
}
