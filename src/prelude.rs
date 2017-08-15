//! Contains common types that can be glob-imported (`*`) for convenience.

pub use app::{Application, ApplicationBuilder};
pub use display_config::DisplayConfig;
pub use engine::Engine;
pub use config::Config;
pub use error::{Error, Result};
pub use event::*;
pub use state::{State, Trans};
