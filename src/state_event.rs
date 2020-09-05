use winit::Event;

use crate::{
    core::{
        ecs::*,
        shrev::{EventChannel, ReaderId},
        EventReader,
    },
    derive::EventReader,
    input::InputEvent,
};

/// The enum holding the different types of event that can be received in a `State` in the
/// `handle_event` method.
#[derive(Clone, Debug, EventReader)]
#[reader(StateEventReader)]
pub enum StateEvent {
    /// Events sent by the winit window.
    Window(Event),
    /// Events sent by the ui system.
    #[cfg(feature = "ui")]
    Ui(ui::UiEvent),
    /// Events sent by the input system.
    Input(InputEvent),
}
