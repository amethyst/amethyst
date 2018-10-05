use renderer::Event;
use ui::UiEvent;
use core::EventReader;
use core::shrev::{ReaderId, EventChannel};
use core::specs::{World, Read};

/// The enum holding the different types of event that can be received in a `State` in the handle_event method.
#[derive(Clone)]
pub enum StateEvent {
    /// Events sent by the winit window.
    Window(Event),
    /// Events sent by the ui system.
    Ui(UiEvent),
}

pub struct StateEventReader {
    window: ReaderId<Event>,
    ui: ReaderId<UiEvent>,
}

impl StateEventReader {
    pub fn new(world: &mut World) -> Self {
        let window = world
            .write_resource::<EventChannel<Event>>()
            .register_reader();
        let ui = world
            .write_resource::<EventChannel<UiEvent>>()
            .register_reader();

        StateEventReader {
            window, ui
        }
    }
}

impl<'a> EventReader<'a> for StateEventReader {
    type SystemData = (Read<'a, EventChannel<Event>>, Read<'a, EventChannel<UiEvent>>);
    type Event = StateEvent;

    fn read(&mut self, data: Self::SystemData, events: &mut Vec<StateEvent>) {
        events.extend(data.0.read(&mut self.window).cloned().map(|e| StateEvent::Window(e)));
        events.extend(data.1.read(&mut self.ui).cloned().map(|e| StateEvent::Ui(e)));
    }

    fn build(world: &mut World) -> Self {
        StateEventReader {
            window: world.write_resource::<EventChannel<Event>>().register_reader(),
            ui: world.write_resource::<EventChannel<UiEvent>>().register_reader(),
        }
    }
}