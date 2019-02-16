use crate::{
    core::{
        shrev::{EventChannel, ReaderId},
        specs::{Read, Resources, SystemData},
        EventReader,
    },
    derive::EventReader,
    renderer::Event,
    ui::UiEvent,
};

/// The enum holding the different types of event that can be received in a `State` in the handle_event method.
#[derive(Clone, EventReader)]
#[reader(StateEventReader)]
pub enum StateEvent {
    /// Events sent by the winit window.
    Window(Event),
    /// Events sent by the ui system.
    Ui(UiEvent),
}
