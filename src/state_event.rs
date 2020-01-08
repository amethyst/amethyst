use derivative::Derivative;
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
#[derive(Debug, Derivative, EventReader)]
#[derivative(Clone(bound = ""))]
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
