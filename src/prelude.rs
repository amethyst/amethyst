//! Contains common types that can be glob-imported (`*`) for convenience.

pub use crate::{
    app::{Application, ApplicationBuilder, CoreApplication, CoreApplicationBuilder},
    callback_queue::{Callback, CallbackQueue},
    config::Config,
    core::{SystemExt, WithNamed},
    ecs::prelude::{Builder, World},
    game_data::{DataInit, GameData, GameDataBuilder},
    state::{GlobalCallback, State, StateCallback, StateError, StateStorage, States},
    state_event::StateEvent,
    trans::{Trans, TransEvent},
};
