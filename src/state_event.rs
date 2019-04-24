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
#[derive(Clone, EventReader)]
#[reader(StateEventReader)]
pub enum StateEvent<T = String>
where
    T: Clone + Send + Sync + 'static,
{
    /// Events sent by the winit window.
    Window(Event),
    /// Events sent by the ui system.
    Ui(UiEvent),
    /// Events sent by the input system.
    Input(InputEvent<T>),
}
