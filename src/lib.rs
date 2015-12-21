#![crate_name = "amethyst"]
#![crate_type = "lib"]

//! Amethyst is a free and open source SDK (software development kit) written in
//! idiomatic [Rust](https://www.rust-lang.org/) for building video games and
//! interactive multimedia applications.
//!
//! # Example
//!
//! ```
//! extern crate amethyst;
//!
//! use amethyst::core::{Duration, Event, Key};
//! use amethyst::engine::{Application, Game, State};
//!
//! struct GameState;
//!
//! impl State for GameState {
//!     fn new() -> GameState {
//!         GameState
//!     }
//!
//!     fn handle_events(&mut self, game: &Game, events: &Vec<Event>) {
//!         for e in events {
//!             match e {
//!                 Event::Closed => game.quit(),
//!                 Event::Resized(x, y) => println!("x: {}, y: {}", x, y),
//!                 Event::KeyPressed(k) => if k == Key::Esc { game.quit() }
//!             }
//!         }
//!     }
//!
//!     fn update(&mut self, game: &Game, delta: Duration) {
//!         println!("Computing some more whoop-ass...");
//!     }
//! }
//!
//! fn main() {
//!     let mut game = Application::new(GameState::new());
//!     game.run();
//! }
//! ```

macro_rules! benchmark {
    ($function:expr) => {{
        let mut timer = ::core::Stopwatch::new();
        timer.restart();
        $function;
        timer.stop();
        timer.elapsed()
    }}
}

pub mod core;
pub mod engine;
pub mod renderer;
