//! Contains common types that can be glob-imported (`*`) for convenience.

pub use crate::{
    app::{Application, ApplicationBuilder, CoreApplication},
    callback_queue::{Callback, CallbackQueue},
    config::Config,
    core::{SystemExt, WithNamed},
    ecs::prelude::{Builder, World},
    game_data::{DataInit, GameData, GameDataBuilder},
    state::{
        EmptyState, EmptyTrans, SimpleState, SimpleTrans, State, StateData, Trans, TransEvent,
    },
    state_event::StateEvent,
};
