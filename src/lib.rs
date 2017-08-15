//! Amethyst is a free and open source game engine written in idiomatic
//! [Rust][rs] for building video games and interactive multimedia applications.
//! The source code is available for download on [GitHub][gh]. See the
//! [online book][bk] for a complete guide to using Amethyst.
//!
//! [rs]: https://www.rust-lang.org/
//! [gh]: https://github.com/amethyst/amethyst
//! [bk]: https://www.amethyst.rs/book/
//!
//! This project is a work in progress and is very incomplete. Pardon the dust!
//!
//! # Example
//!
//! ```rust,no_run
//! extern crate amethyst;
//!
//! use amethyst::prelude::*;
//!
//! struct GameState;
//!
//! impl State for GameState {
//!     fn on_start(&mut self, _: &mut Engine) {
//!         println!("Starting game!");
//!     }
//!
//!     fn handle_event(&mut self, _: &mut Engine, event: &Event) -> Trans {
//!         match event {
//!             Event::Window(e) => match e {
//!                 WindowEvent::KeyboardInput(_, _, Some(Key::Escape), _) |
//!                 WindowEvent::Closed => Trans::Quit,
//!                 _ => Trans::None,
//!             }
//!             _ => Trans::None,
//!         }
//!     }
//!
//!     fn update(&mut self, _: &mut Engine) -> Trans {
//!         println!("Computing some more whoop-ass...");
//!     }
//! }
//!
//! fn main() {
//!     let path = format!("{}/examples/01_window/resources/config.ron",
//!                        env!("CARGO_MANIFEST_DIR"));
//!     let cfg = DisplayConfig::load(path);
//!     let mut game = Application::build(GameState, cfg).done();
//!     game.run();
//! }
//! ```

#![deny(missing_docs)]
#![doc(html_logo_url = "https://tinyurl.com/jtmm43a")]

pub extern crate amethyst_assets as assets;
#[macro_use]
pub extern crate amethyst_config as config;
pub extern crate amethyst_renderer as renderer;

extern crate cgmath;
extern crate dds;
#[macro_use]
extern crate derivative;
extern crate fnv;
extern crate genmesh;
extern crate imagefmt;
extern crate num_cpus;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate smallvec;
extern crate specs;
extern crate wavefront_obj;
extern crate winit;

#[cfg(feature="profiler")]
#[macro_use]
extern crate thread_profiler;

pub use self::app::{Application, ApplicationBuilder};
pub use self::engine::Engine;
pub use self::error::{Error, Result};
pub use self::state::{State, StateMachine, Trans};

pub mod ecs;
pub mod event;
pub mod prelude;
pub mod timing;

mod app;
mod display_config;
mod engine;
mod state;
mod error;
