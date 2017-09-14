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
//!     fn handle_event(&mut self, _: &mut Engine, event: Event) -> Trans {
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
//!     fn update(&mut self, _: &mut Engine) -> Trans {
//!         println!("Computing some more whoop-ass...");
//!         Trans::Quit
//!     }
//! }
//!
//! fn main() {
//!     let mut game = Application::new(GameState).expect("Fatal error");

//!     game.run();
//! }
//! ```

#![deny(missing_docs)]
#![doc(html_logo_url = "https://tinyurl.com/jtmm43a")]

#[macro_use]
#[cfg(feature = "profiler")]
pub extern crate thread_profiler;

pub extern crate amethyst_config as config;
pub extern crate amethyst_renderer as renderer;
pub extern crate amethyst_input as input;

extern crate amethyst_assets;
extern crate cgmath;
extern crate crossbeam;
#[macro_use]
extern crate derivative;
extern crate fnv;
extern crate futures;
extern crate gfx;
extern crate imagefmt;
extern crate num_cpus;
extern crate rayon;
extern crate rodio;
extern crate smallvec;
extern crate shred;
extern crate specs;
extern crate wavefront_obj;
extern crate winit;

pub use self::app::{Application, ApplicationBuilder};
pub use self::engine::Engine;
pub use self::error::{Error, Result};
pub use self::state::{State, StateMachine, Trans};

pub mod assets;
pub mod audio;
pub mod ecs;
pub mod event;
pub mod prelude;
pub mod timing;

mod app;
mod engine;
mod state;
mod error;
