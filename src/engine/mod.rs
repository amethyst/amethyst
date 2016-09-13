//! Game engine sitting atop the core libraries.

mod app;
mod state;
mod config;

pub use self::app::{Application, ApplicationBuilder};
pub use self::state::{State, StateMachine, Trans};
pub use self::config::Config;

use ecs;
use context::Context;
use std::sync::{Arc, Mutex};

pub type Planner = ecs::Planner<Arc<Mutex<Context>>>;
