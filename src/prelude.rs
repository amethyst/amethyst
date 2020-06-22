//! Contains common types that can be glob-imported (`*`) for convenience.

pub use crate::{
    app::{Application, ApplicationBuilder, CoreApplication},
    config::Config,
    ecs::prelude::*,
    game_data::{DataInit, GameData, GameDataBuilder},
    state::{
        EmptyState, EmptyTrans, SimpleState, SimpleTrans, State, StateData, Trans, TransEvent,
    },
    state_event::StateEvent,
};
