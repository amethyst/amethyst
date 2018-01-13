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
//! use amethyst::renderer::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
//!
//! struct GameState;
//!
//! impl State for GameState {
//!     fn on_start(&mut self, _: &mut World) {
//!         println!("Starting game!");
//!     }
//!
//!     fn handle_event(&mut self, _: &mut World, event: Event) -> Trans {
//!         match event {
//!             Event::WindowEvent { event, .. } => match event {
//!                 WindowEvent::KeyboardInput {
//!                     input: KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Escape), .. }, ..
//!                 } |
//!                 WindowEvent::Closed => Trans::Quit,
//!                 _ => Trans::None,
//!             },
//!             _ => Trans::None,
//!         }
//!     }
//!
//!     fn update(&mut self, _: &mut World) -> Trans {
//!         println!("Computing some more whoop-ass...");
//!         Trans::Quit
//!     }
//! }
//!
//! fn main() {
//!     let mut game = Application::new("assets/", GameState).expect("Fatal error");
//!     game.run();
//! }
//! ```

#![deny(missing_docs)]
#![doc(html_logo_url = "https://tinyurl.com/jtmm43a")]

#[macro_use]
#[cfg(feature = "profiler")]
pub extern crate thread_profiler;

pub extern crate amethyst_assets as assets;
pub extern crate amethyst_audio as audio;
pub extern crate amethyst_config as config;
pub extern crate amethyst_core as core;
pub extern crate amethyst_input as input;
pub extern crate amethyst_renderer as renderer;
pub extern crate amethyst_ui as ui;
pub extern crate amethyst_utils as utils;
pub extern crate shred;
pub extern crate shrev;
pub extern crate specs as ecs;
pub extern crate winit;

#[macro_use]
extern crate derivative;
#[macro_use]
extern crate log;
extern crate rayon;
extern crate rustc_version_runtime;

pub use self::app::{Application, ApplicationBuilder};
pub use self::error::{Error, Result};
pub use self::state::{State, StateMachine, Trans};

pub mod prelude;

mod app;
mod error;
mod state;
mod vergen;
mod bundle;
