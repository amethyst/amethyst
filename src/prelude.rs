//! Contains common types that can be glob-imported (`*`) for convenience.

pub use app::{Application, ApplicationBuilder};
pub use config::Config;
pub use ecs::prelude::{Builder, World};
pub use game_data::{DataInit, GameData, GameDataBuilder};
pub use state::{EmptyState, EmptyTrans, SimpleState, SimpleTrans, State, StateData, Trans};
pub use state_event::StateEvent;
pub use core::WithNamed;
