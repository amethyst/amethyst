use shrev::{EventChannel, ReaderId};
use specs::{Component, DenseVecStorage, Fetch, Join, System, WriteStorage};
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
    event_reader: ReaderId,
}

impl ResizeSystem {
    /// Creates a new ResizeSystem that listens with the given reader Id.
    pub fn new(winit_event_reader: ReaderId) -> ResizeSystem {
        ResizeSystem {
            event_reader: winit_event_reader,
        }
    }
}

impl<'a> System<'a> for ResizeSystem {
    type SystemData = (
        WriteStorage<'a, UiTransform>,
        WriteStorage<'a, UiResize>,
        Fetch<'a, EventChannel<Event>>,
    );

    fn run(&mut self, (mut transform, mut resize, events): Self::SystemData) {
        for event in events
            .lossy_read(&mut self.event_reader)
            .expect("ResizeSystem failed!")
        {
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
}
