//! # amethyst_events
//!
//! The standard events used in State::handle_event.
#![warn(missing_docs)]

extern crate amethyst_renderer;
extern crate amethyst_ui;

pub use self::state_event::StateEvent;

mod state_event;