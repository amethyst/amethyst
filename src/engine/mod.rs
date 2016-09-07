//! Game engine sitting atop the core libraries.

mod app;
mod state;

pub use self::app::{Application, ApplicationBuilder};
pub use self::state::{State, StateMachine, Trans};
