use amethyst::{
    ecs::System,
    shrev::{EventChannel, ReaderId},
    ui::UiEvent,
};

/// This shows how to handle UI events. This is the same as in the 'ui' example.
pub struct UiEventHandlerSystem {
    reader_id: ReaderId<UiEvent>,
}

impl UiEventHandlerSystem {
    pub fn new(reader_id: ReaderId<UiEvent>) -> Self {
        Self { reader_id }
    }
}

impl<'a> System for UiEventHandlerSystem {
    type SystemData = Write<'a, EventChannel<UiEvent>>;

    fn run(&mut self, events: Self::SystemData) {
        // Reader id was just initialized above if empty
        for ev in events.read(&mut self.reader_id) {
            log::info!("[SYSTEM] You just interacted with an ui element: {:?}", ev);
        }
    }
}
