#![crate_name = "amethyst_engine"]
#![crate_type = "lib"]
#![doc(html_logo_url = "http://tinyurl.com/hgsb45k")]

//! Game engine sitting atop the core libraries.

mod app;
mod state;
mod timing;

pub use self::app::Application;
pub use self::state::{State, StateMachine, Trans};
pub use self::timing::{Duration, Instant, Stopwatch};
