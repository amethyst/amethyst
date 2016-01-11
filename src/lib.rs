#![crate_name = "amethyst"]
#![crate_type = "lib"]

//! Amethyst is a free and open source game engine written in idiomatic
//! [Rust][rs] for building video games and interactive multimedia applications.
//! The source code is available for download on [GitHub][gh].
//!
//! [rs]: https://www.rust-lang.org/
//! [gh]: https://github.com/ebkalderon/amethyst
//!
//! This project is a work in progress and is very incomplete. Pardon the dust!
//!
//! # Example
//!
//! ```ignore
//! extern crate amethyst;
//!
//! use amethyst::*;
//!
//! struct GameState;
//!
//! impl State for GameState {
//!     fn handle_events(&mut self, events: &Vec<Event>) -> Trans {
//!         for e in events {
//!             match e {
//!                 Event::Closed => Trans::Quit,
//!                 Event::Resized(x, y) => println!("x: {}, y: {}", x, y),
//!                 Event::KeyPressed(k) => if k == Key::Esc { Trans::Quit },
//!             }
//!         }
//!         Trans::None
//!     }
//!
//!     fn update(&mut self, _delta: Duration) -> Trans {
//!         println!("Computing some more whoop-ass...");
//!         Trans::None
//!     }
//! }
//!
//! fn main() {
//!     let mut game = Application::new(GameState);
//!     game.run();
//! }
//! ```

mod engine;
pub mod renderer;

pub use self::engine::app::Application;
pub use self::engine::state::{State, StateMachine, Trans};
pub use self::engine::timing::{Duration, Stopwatch};
