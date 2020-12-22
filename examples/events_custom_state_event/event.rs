use amethyst::{
    core::{
        shrev::{EventChannel, ReaderId},
        EventReader,
    },
    derive::EventReader,
    ecs::Resources,
};
use amethyst_input::InputEvent;

/// Here's a copy of the original StateEvent with our own type added
#[derive(Clone, Debug, EventReader)]
#[reader(MyExtendedStateEventReader)]
pub enum MyExtendedStateEvent {
    /// Events sent by the winit window.
    Window(winit::event::Event<'static, ()>),
    /// Events sent by the input system.
    Input(InputEvent),
    /// Our own events for our own game logic
    Game(GameEvent),
}

#[derive(Clone, Debug, PartialEq)]
pub enum GameEvent {
    IncreaseDifficulty,
}
