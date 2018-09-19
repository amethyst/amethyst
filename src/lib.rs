//! Amethyst is a free and open source game engine written in idiomatic
//! [Rust][rs] for building video games and interactive multimedia applications.
//! The source code is available for download on [GitHub][gh]. See the
//! [online book][bk] for a complete guide to using Amethyst.
//!
//! [rs]: https://www.rust-lang.org/
//! [gh]: https://github.com/amethyst/amethyst
//! [bk]: https://www.amethyst.rs/book/master/
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
//! impl EmptyState for GameState {
//!     fn on_start(&mut self, _: StateData<()>) {
//!         println!("Starting game!");
//!     }
//!
//!     fn handle_event(&mut self, _: StateData<()>, event: StateEvent<()>) -> EmptyTrans {
//!         if let StateEvent::Window(event) = &event {
//!             match event {
//!                  Event::WindowEvent { event, .. } => match event {
//!                     WindowEvent::KeyboardInput {
//!                         input: KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Escape), .. }, ..
//!                     } |
//!                     WindowEvent::CloseRequested => Trans::Quit,
//!                     _ => Trans::None,
//!                 },
//!                 _ => Trans::None,
//!             }
//!         } else {
//!             Trans::None
//!         }
//!     }
//!
//!     fn update(&mut self, _: StateData<()>) -> EmptyTrans {
//!         println!("Computing some more whoop-ass...");
//!         Trans::Quit
//!     }
//! }
//!
//! fn main() {
//!     let mut game = Application::new("assets/", GameState, ()).expect("Fatal error");
//!     game.run();
//! }
//! ```

#![warn(missing_docs)]
#![doc(html_logo_url = "https://www.amethyst.rs/assets/amethyst.svg")]

#[macro_use]
#[cfg(feature = "profiler")]
pub extern crate thread_profiler;

pub extern crate amethyst_animation as animation;
pub extern crate amethyst_assets as assets;
pub extern crate amethyst_audio as audio;
pub extern crate amethyst_config as config;
pub extern crate amethyst_controls as controls;
pub extern crate amethyst_core as core;
pub extern crate amethyst_input as input;
pub extern crate amethyst_locale as locale;
pub extern crate amethyst_renderer as renderer;
pub extern crate amethyst_ui as ui;
pub extern crate amethyst_utils as utils;
pub extern crate winit;

extern crate amethyst_ui;
#[macro_use]
extern crate derivative;
extern crate fern;
#[macro_use]
extern crate log;
extern crate amethyst_input;
extern crate rayon;
extern crate rustc_version_runtime;

pub use self::app::{Application, ApplicationBuilder};
pub use self::error::{Error, Result};
pub use self::game_data::{DataInit, GameData, GameDataBuilder};
pub use self::logger::{start_logger, LevelFilter as LogLevelFilter, LoggerConfig};
pub use self::state::{
    EmptyState, EmptyTrans, SimpleState, SimpleTrans, State, StateData, StateMachine, Trans,
};
pub use self::state_event::StateEvent;
pub use core::shred;
pub use core::shrev;
pub use core::specs as ecs;

pub mod prelude;

mod app;
mod error;
mod game_data;
mod logger;
mod state;
mod state_event;
