use derivative::Derivative;
use winit::Event;

use crate::{
    core::shrev::EventChannel,
    input::{BindingTypes, InputEvent, StringBindings},
};

/// The enum holding the different types of event that can be received in a `State` in the
/// `handle_event` method.
#[derive(Debug, Derivative)]
#[derivative(Clone(bound = ""))]
pub enum StateEvent<T = StringBindings>
where
    T: BindingTypes,
{
    /// Events sent by the winit window.
    Window(Event),
    // /// Events sent by the ui system.
    // Ui(UiEvent),
    /// Events sent by the input system.
    Input(InputEvent<T>),
}

/// Predefined event channel that holds [StateEvent]
pub type StateEventChannel<T = StringBindings> = EventChannel<StateEvent<T>>;
