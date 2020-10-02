//! Contains common types that can be glob-imported (`*`) for convenience.

#[doc(no_inline)]
pub use crate::{
    app::{Application, ApplicationBuilder, CoreApplication},
    callback_queue::{Callback, CallbackQueue},
    config::Config,
    core::{SystemDesc, SystemExt, WithNamed},
    ecs::prelude::{Builder, World, WorldExt},
    game_data::{DataInit, GameData, GameDataBuilder},
    state::{
        EmptyState, EmptyTrans, SimpleState, SimpleTrans, State, StateData, Trans, TransEvent,
    },
    state_event::StateEvent,
};
