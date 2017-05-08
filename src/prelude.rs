//! Contains common types that can be glob-imported (`*`) for convenience.

pub use app::{Application, ApplicationBuilder, Engine};
pub use config::{Config, DisplayConfig, Element};
pub use error::{Error, Result};
pub use event::*;
pub use state::{State, Trans};
