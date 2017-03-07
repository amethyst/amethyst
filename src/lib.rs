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
//! ```no_run
//! extern crate amethyst;
//!
//! use amethyst::{Application, Event, State, Trans, VirtualKeyCode, WindowEvent};
//! use amethyst::asset_manager::AssetManager;
//! use amethyst::config::Config;
//! use amethyst::ecs::World;
//! use amethyst::gfx_device::DisplayConfig;
//! use amethyst::renderer::Pipeline;
//!
//! struct GameState;
//!
//! impl State for GameState {
//!     fn on_start(&mut self, _: &mut World, _: &mut AssetManager, pipe: &mut Pipeline) {
//!         use amethyst::renderer::pass::Clear;
//!         use amethyst::renderer::Layer;
//!         let clear_layer = Layer::new("main", vec![
//!             Clear::new([0.0, 0.0, 0.0, 1.0]),
//!         ]);
//!         pipe.layers.push(clear_layer);
//!     }
//!
//!     fn handle_events(&mut self,
//!                      events: &[WindowEvent],
//!                      _: &mut World,
//!                      _: &mut AssetManager,
//!                      _: &mut Pipeline) -> Trans {
//!         for e in events {
//!             match e.payload {
//!                 Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return Trans::Quit,
//!                 Event::Closed => return Trans::Quit,
//!                 _ => (),
//!             }
//!         }
//!         Trans::None
//!     }
//! }
//!
//! fn main() {
//!     let path = format!("{}/examples/01_window/resources/config.yml",
//!                        env!("CARGO_MANIFEST_DIR"));
//!     let cfg = DisplayConfig::load(path);
//!     let mut game = Application::build(GameState, cfg).done();
//!     game.run();
//! }
//! ```

#![crate_name = "amethyst"]
#![crate_type = "lib"]
#![deny(missing_docs)]
#![doc(html_logo_url = "https://tinyurl.com/jtmm43a")]

#[macro_use]
#[cfg(feature="profiler")]
pub extern crate thread_profiler;
pub extern crate amethyst_renderer as renderer;
#[macro_use]
pub extern crate amethyst_config as config;

extern crate cgmath;
extern crate dds;
extern crate fnv;
extern crate gfx;
extern crate genmesh;
extern crate glutin;
extern crate imagefmt;
extern crate num_cpus;
extern crate rayon;
extern crate smallvec;
extern crate specs;
extern crate wavefront_obj;

pub mod asset_manager;
pub mod ecs;

mod engine;

pub use engine::*;
