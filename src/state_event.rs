use crate::{
    core::{
        ecs::{Read, Resources, SystemData},
        shrev::{EventChannel, ReaderId},
        EventReader,
    },
    derive::EventReader,
    input::InputEvent,
    renderer::Event,
    ui::UiEvent,
};

/// The enum holding the different types of event that can be received in a `State` in the handle_event method.
/// This assumes that you used `String` as the identifier for the `InputBundle`. If this is not the
/// case, you will want to implement your own StateEvent.
#[derive(Clone, EventReader)]
#[reader(StateEventReader)]
pub enum StateEvent {
    /// Events sent by the winit window.
    Window(Event),
    /// Events sent by the ui system.
    Ui(UiEvent),
    Input(InputEvent::<String>),
}
