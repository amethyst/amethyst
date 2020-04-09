use amethyst::{
    core::{
        shrev::{EventChannel, ReaderId},
        EventReader,
    },
    derive::EventReader,
    ecs::{Read, SystemData, World},
};
use amethyst_input::{BindingTypes, InputEvent, StringBindings};
use amethyst_ui::UiEvent;
use winit::Event;

/// Here's a copy of the original StateEvent with our own type added
#[derive(Clone, Debug, EventReader)]
#[reader(MyExtendedStateEventReader)]
pub enum MyExtendedStateEvent<T = StringBindings>
where
    T: BindingTypes + Clone,
{
    /// Events sent by the winit window.
    Window(Event),
    /// Events sent by the ui system.
    Ui(UiEvent),
    /// Events sent by the input system.
    Input(InputEvent<T>),
    /// Our own events for our own game logic
    Game(GameEvent),
}

#[derive(Clone, Debug, PartialEq)]
pub enum GameEvent {
    IncreaseDifficulty,
}
