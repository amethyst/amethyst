#![crate_name = "amethyst"]
#![crate_type = "lib"]
#![doc(html_logo_url = "http://tinyurl.com/jtmm43a")]

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
//! ```ignore
//! extern crate amethyst;
//!
//! use amethyst::*;
//!
//! struct GameState;
//!
//! impl State for GameState {
//!     fn handle_events(&mut self, events: &[Event]) -> Trans {
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

pub mod engine;

pub extern crate amethyst_ecs;
pub extern crate amethyst_renderer;
pub extern crate amethyst_config;
pub extern crate amethyst_context;

pub use amethyst_ecs as ecs;
pub use amethyst_renderer as renderer;
pub use amethyst_config as config;
pub use amethyst_context as context;
