//! Game engine sitting atop the core libraries.

pub mod app;
pub mod state;

pub use self::app::Application;
pub use self::state::{State, StateMachine};

