//! Game engine sitting atop the core libraries.

pub mod app;
pub mod state;
pub mod timing;

pub use self::app::Application;
pub use self::state::{State, StateMachine, Trans};
pub use self::timing::{Duration, SteadyTime, Stopwatch};
