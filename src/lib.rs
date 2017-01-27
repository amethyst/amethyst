#![crate_name = "amethyst"]
#![crate_type = "lib"]
#![doc(html_logo_url = "https://tinyurl.com/jtmm43a")]

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
//! # Example
//!
//! ```ignore
//! extern crate amethyst;
//!
//! use amethyst::engine::{Application, State, Trans};
//! use amethyst::config::Element;
//! use amethyst::specs::World;
//! use amethyst::gfx_device::DisplayConfig;
//! use amethyst::asset_manager::AssetManager;
//! use amethyst::event::WindowEvent;
//! use amethyst::renderer::Pipeline;
//!
//! struct GameState;
//!
//! impl State for GameState {
//!     fn on_start(&mut self, _: &mut World, _: &mut AssetManager, pipeline: &mut Pipeline) {
//!         use amethyst::renderer::pass::Clear;
//!         use amethyst::renderer::Layer;
//!         let clear_layer =
//!             Layer::new("main",
//!                     vec![
//!                         Clear::new([0.0, 0.0, 0.0, 1.0]),
//!                     ]);
//!         pipeline.layers = vec![clear_layer];
//!     }
//!
//!     fn handle_events(&mut self, events: &[WindowEvent], _: &mut World, _: &mut AssetManager, _: &mut Pipeline) -> Trans {
//!         use amethyst::event::*;
//!         for event in events {
//!             match event.payload {
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
//!     let display_config = DisplayConfig::from_file(path).unwrap();
//!     let mut game = Application::build(GameState, display_config).done();
//!     game.run();
//! }
//! ```

pub mod world_resources;
pub mod engine;
pub mod systems;
pub mod components;
pub mod gfx_device;
pub mod asset_manager;
pub mod event;

#[macro_use]
pub extern crate amethyst_config as config;
pub extern crate amethyst_renderer as renderer;
pub extern crate specs as specs;
