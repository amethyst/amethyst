//! Amethyst is a free and open source game engine written in idiomatic
//! [Rust][rs] for building video games and interactive multimedia applications.
//! The source code is available for download on [GitHub][gh]. See the
//! [online book][bk] for a complete guide to using Amethyst.
//!
//! [rs]: https://www.rust-lang.org/
//! [gh]: https://github.com/amethyst/amethyst
//! [bk]: https://book.amethyst.rs/master/
//!
//! This project is a work in progress and is very incomplete. Pardon the dust!
//!
//! # Example
//!
//! ```rust,no_run
//! use amethyst::prelude::*;
//! use amethyst::winit::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
//!
//! struct GameState;
//!
//! impl SimpleState for GameState {
//!     fn on_start(&mut self, _: StateData<'_, GameData<'_, '_>>) {
//!         println!("Starting game!");
//!     }
//!
//!     fn handle_event(&mut self, _: StateData<'_, GameData<'_, '_>>, event: StateEvent) -> SimpleTrans {
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
//!     fn update(&mut self, _: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
//!         println!("Computing some more whoop-ass...");
//!         Trans::Quit
//!     }
//! }
//!
//! fn main() {
//!     let mut game = Application::new("assets/", GameState, GameDataBuilder::default())
//!         .expect("Fatal error");
//!     game.run();
//! }
//! ```

#![doc(html_logo_url = "https://amethyst.rs/brand/logo-standard.svg")]
#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

#[cfg(feature = "animation")]
pub use amethyst_animation as animation;
pub use amethyst_assets as assets;
#[cfg(feature = "audio")]
pub use amethyst_audio as audio;
pub use amethyst_config as config;
pub use amethyst_controls as controls;
pub use amethyst_core as core;
pub use amethyst_derive as derive;
pub use amethyst_error as error;
#[cfg(feature = "gltf")]
pub use amethyst_gltf as gltf;
pub use amethyst_input as input;
#[cfg(feature = "locale")]
pub use amethyst_locale as locale;
#[cfg(feature = "network")]
pub use amethyst_network as network;
pub use amethyst_rendy as renderer;
pub use amethyst_ui as ui;
pub use amethyst_utils as utils;
pub use amethyst_window as window;
pub use winit;

pub use crate::core::{ecs, shred, shrev};
#[doc(hidden)]
pub use crate::derive::*;

pub use self::{
    app::{Application, ApplicationBuilder, CoreApplication},
    callback_queue::{Callback, CallbackQueue},
    error::Error,
    game_data::{DataDispose, DataInit, GameData, GameDataBuilder},
    logger::{start_logger, LevelFilter as LogLevelFilter, Logger, LoggerConfig, StdoutLog},
    state::{
        EmptyState, EmptyTrans, SimpleState, SimpleTrans, State, StateData, StateMachine, Trans,
        TransEvent,
    },
    state_event::{StateEvent, StateEventReader},
};

/// Convenience alias for use in main functions that uses Amethyst.
pub type Result<T> = std::result::Result<T, error::Error>;

pub mod prelude;

mod app;
mod callback_queue;
mod game_data;
mod logger;
mod state;
mod state_event;
