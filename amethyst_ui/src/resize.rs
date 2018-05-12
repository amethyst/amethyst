use amethyst_core::specs::prelude::{Component, DenseVecStorage, Join, Read, Resources, System,
                                    WriteStorage};
use amethyst_core::shrev::{EventChannel, ReaderId};
use winit::{Event, WindowEvent};

use super::*;

/// Whenever the window is resized the function in this component will be called on this
/// entity's UiTransform, along with the new width and height of the window.
pub struct UiResize(pub Box<FnMut(&mut UiTransform, (f32, f32)) + Send + Sync>);

impl Component for UiResize {
    type Storage = DenseVecStorage<Self>;
}

/// This system rearranges UI elements whenever the screen is resized using their `UiResize`
/// component.
pub struct ResizeSystem {
    event_reader: Option<ReaderId<Event>>,
}

impl ResizeSystem {
    /// Creates a new ResizeSystem that listens with the given reader Id.
    pub fn new() -> ResizeSystem {
        ResizeSystem { event_reader: None }
    }
}

impl<'a> System<'a> for ResizeSystem {
    type SystemData = (
        WriteStorage<'a, UiTransform>,
        WriteStorage<'a, UiResize>,
        Read<'a, EventChannel<Event>>,
    );

    fn run(&mut self, (mut transform, mut resize, events): Self::SystemData) {
        for event in events.read(&mut self.event_reader.as_mut().unwrap()) {
            if let &Event::WindowEvent {
                event: WindowEvent::Resized(width, height),
                ..
            } = event
            {
                for (transform, resize) in (&mut transform, &mut resize).join() {
                    (resize.0)(transform, (width as f32, height as f32));
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::specs::prelude::SystemData;
        Self::SystemData::setup(res);
        self.event_reader = Some(res.fetch_mut::<EventChannel<Event>>().register_reader());
    }
}
