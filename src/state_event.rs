use winit::event::Event;

use crate::{
    core::{
        ecs::{Read, SystemData, World},
        shrev::{EventChannel, ReaderId},
        EventReader,
    },
    derive::EventReader,
    input::{BindingTypes, InputEvent, StringBindings},
    ui::UiEvent,
};

/// The enum holding the different types of event that can be received in a `State` in the
/// `handle_event` method.
#[derive(Debug, EventReader)]
#[reader(StateEventReader)]
pub enum StateEvent<T = StringBindings>
where
    T: BindingTypes,
{
    /// Events sent by the winit window.
    Window(Event<'static, ()>),
    /// Events sent by the ui system.
    Ui(UiEvent),
    /// Events sent by the input system.
    Input(InputEvent<T>),
}

impl<T> Clone for StateEvent<T>
where
    T: BindingTypes,
{
    fn clone(&self) -> Self {
        match self {
            Self::Window(e) => Self::Window(e.clone()),
            Self::Ui(ui_event) => Self::Ui(ui_event.clone()),
            Self::Input(input_event) => Self::Input(input_event.clone()),
        }
    }
}
