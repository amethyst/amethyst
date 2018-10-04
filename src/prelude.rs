//! Contains common types that can be glob-imported (`*`) for convenience.

pub use app::{Application, ApplicationBuilder};
pub use config::Config;
pub use core::WithNamed;
pub use ecs::prelude::{Builder, World};
pub use {DataInit, GameData, GameDataBuilder};
pub use state::{EmptyState, EmptyTrans, SimpleState, SimpleTrans, State, StateData, Trans};
pub use events::StateEvent;
