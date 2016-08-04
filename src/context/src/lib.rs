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
pub extern crate rodio;

use amethyst_config::Element;
use std::path::Path;

pub mod video_context;
pub mod broadcaster;
pub mod timing;
pub mod event;
pub mod renderer;
pub mod asset_manager;
mod video_init;
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
    pub renderer: Renderer,
    pub asset_manager: AssetManager,
    pub broadcaster: Broadcaster,
    pub audio_sink: Option<rodio::Sink>,
    pub delta_time: Duration,
    pub fixed_step: Duration,
    pub last_fixed_update: SteadyTime,
}

unsafe impl Send for Context {}

impl Context {
    /// Create a `Context` configured according to `Config`
    pub fn new(config: Config) -> Context {
        let (video_context, factory_impl) =
            video_init::create_video_context_and_factory_impl(config.display_config);
        let renderer = Renderer::new(video_context);
        let asset_manager = AssetManager::new(factory_impl);
        let mut broadcaster = Broadcaster::new();
        broadcaster.register::<EngineEvent>();

        let audio_sink = match rodio::get_default_endpoint() {
            Some(endpoint) => Some(rodio::Sink::new(&endpoint)),
            None => None,
        };

        Context {
            renderer: renderer,
            asset_manager: asset_manager,
            broadcaster: broadcaster,
            audio_sink: audio_sink,
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
        match *video_context {
            VideoContext::OpenGL { ref window, .. } =>
                for event in window.poll_events() {
                    let event = EngineEvent::new(event);
                    events.push(event);
                },
            #[cfg(windows)]
            VideoContext::Direct3D {  } => {
                // stub
                unimplemented!();
            },
            VideoContext::Null => (),
        }
        events
    }
}
