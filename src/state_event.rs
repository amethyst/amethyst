use renderer::Event;
use ui::UiEvent;
use core::EventReader;
use core::shrev::{ReaderId, EventChannel};
use core::specs::{World, Read};

/// The enum holding the different types of event that can be received in a `State` in the handle_event method.
#[derive(Clone)]
pub enum StateEvent<E: Send + Sync + 'static> {
    /// Events sent by the winit window.
    Window(Event),
    /// Events sent by the ui system.
    Ui(UiEvent),
    /// Custom user events.
    /// To receive events from there, you need to write `E` instances into EventChannel<E>
    Custom(E),
}

pub struct StateEventReader<E: Send + Sync + 'static> {
    window: ReaderId<Event>,
    ui: ReaderId<UiEvent>,
    custom: ReaderId<E>,
}

impl <E: Send + Sync + 'static> StateEventReader<E> {
    pub fn new(world: &mut World) -> Self {
        let window = world
            .write_resource::<EventChannel<Event>>()
            .register_reader();
        let ui = world
            .write_resource::<EventChannel<UiEvent>>()
            .register_reader();
        let custom = world
            .write_resource::<EventChannel<E>>()
            .register_reader();

        StateEventReader {
            window, custom, ui
        }
    }
}

impl<'a, E: Clone + Send + Sync + 'static> EventReader<'a> for StateEventReader<E> {
    type SystemData = (Read<'a, EventChannel<Event>>, Read<'a, EventChannel<UiEvent>>, Read<'a, EventChannel<E>>);
    type Event = StateEvent<E>;

    fn read(&mut self, data: Self::SystemData, events: &mut Vec<StateEvent<E>>) {
        events.extend(data.0.read(&mut self.window).cloned().map(|e| StateEvent::Window(e)));
        events.extend(data.1.read(&mut self.ui).cloned().map(|e| StateEvent::Ui(e)));
        events.extend(data.2.read(&mut self.custom).cloned().map(|e| StateEvent::Custom(e)));
    }
}