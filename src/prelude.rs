//! Contains common types that can be glob-imported (`*`) for convenience.

#[doc(no_inline)]
pub use crate::{
    app::{Application, ApplicationBuilder, CoreApplication},
    config::Config,
    ecs::*,
    ecs as legion,
    game_data::{DataInit, GameData},
    state::{
        EmptyState, EmptyTrans, SimpleState, SimpleTrans, State, StateData, Trans, TransEvent,
    },
    state_event::StateEvent,
};
