//! # amethyst_state
//!
//! State and Transitions between States.
#![warn(missing_docs)]

extern crate amethyst_config;
extern crate amethyst_core;
extern crate amethyst_events;
// TODO: FeatureGate
extern crate amethyst_input;
// TODO: FeatureGate
extern crate amethyst_renderer;
// TODO: FeatureGate
extern crate amethyst_ui;
#[macro_use]
extern crate derivative;

pub use self::state::{
	EmptyState, EmptyTrans, SimpleState, SimpleTrans, State,
	StateData, StateError, StateMachine, Trans,
};
pub use self::game_data::{DataInit, GameData, GameDataBuilder};

mod state;
mod game_data;