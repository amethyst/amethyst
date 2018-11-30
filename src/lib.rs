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
//! impl<S> StateCallback<S, StateEvent> for GameState {
//!     fn on_start(&mut self, _: &mut World) {
//!         println!("Starting game!");
//!     }
//!
//!     fn handle_event(&mut self, _: &mut World, event: &StateEvent) -> Trans<S> {
//!         if let StateEvent::Window(event) = event {
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
//!     fn update(&mut self, _: &mut World) -> Trans<S> {
//!         println!("Computing some more whoop-ass...");
//!         Trans::Quit
//!     }
//! }
//!
//! fn main() -> amethyst::Result<()> {
//!     let mut game = Application::build("assets/")?
//!         .with_state((), GameState)?
//!         .build(GameDataBuilder::default())?;
//!
//!     game.run();
//!     Ok(())
//! }
//! ```

#![doc(html_logo_url = "https://www.amethyst.rs/assets/amethyst.svg")]
#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

#[macro_use]
#[cfg(feature = "profiler")]
pub extern crate thread_profiler;

pub extern crate amethyst_animation as animation;
pub extern crate amethyst_assets as assets;
pub extern crate amethyst_audio as audio;
pub extern crate amethyst_config as config;
pub extern crate amethyst_controls as controls;
pub extern crate amethyst_core as core;
#[macro_use]
pub extern crate amethyst_derive as derive;
pub extern crate amethyst_input as input;
pub extern crate amethyst_locale as locale;
pub extern crate amethyst_network as network;
pub extern crate amethyst_renderer as renderer;
pub extern crate amethyst_ui as ui;
pub extern crate amethyst_utils as utils;
pub extern crate winit;

extern crate crossbeam_channel;
#[macro_use]
extern crate derivative;
extern crate fern;
#[macro_use]
extern crate log;
extern crate amethyst_input;
extern crate rayon;
extern crate rustc_version_runtime;
#[macro_use]
extern crate serde_derive;
extern crate hashbrown;
extern crate smallvec;

pub use crate::core::{shred, shrev, specs as ecs};

pub use self::{
    callback_queue::{Callback, CallbackQueue},
    error::{Error, Result},
    logger::{start_logger, LevelFilter as LogLevelFilter, Logger, LoggerConfig, StdoutLog},
    state_event::{StateEvent, StateEventReader},
};

#[cfg(not(feature = "dynamic_app"))]
pub use self::{
    app::{Application, ApplicationBuilder, CoreApplication},
    game_data::{DataInit, GameData, GameDataBuilder},
    state::{
        EmptyState, EmptyTrans, SimpleState, SimpleTrans, State, StateData, StateMachine, Trans,
        TransEvent,
    },
};

#[cfg(feature = "dynamic_app")]
pub use self::dynamic::*;

#[doc(hidden)]
pub use crate::derive::*;

pub mod prelude;

#[cfg(not(feature = "dynamic_app"))]
mod app;
mod callback_queue;
mod dynamic;
mod error;
mod game_data;
mod logger;
#[cfg(not(feature = "dynamic_app"))]
mod state;
mod state_event;
