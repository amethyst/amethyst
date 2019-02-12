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
/// I is the generic type of virtual input events. See `InputEvent<T>` for more information.
#[derive(Clone, EventReader)]
#[reader(StateEventReader)]
pub enum StateEvent<I> {
    /// Events sent by the winit window.
    Window(Event),
    /// Events sent by the ui system.
    Ui(UiEvent),
    Input(InputEvent::<I>),
}
