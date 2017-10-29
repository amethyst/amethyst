use shrev::{EventChannel, ReaderId};
use specs::{Component, DenseVecStorage, Fetch, Join, System, WriteStorage};
use winit::{Event, WindowEvent};

/// The raw pixels on screen that are populated.
///
/// TODO: Eventually this should be either replaced by a citrine type, or citrine may just
/// populate it.
pub struct UiTransform {
    /// An identifier. Serves no purpose other than to help you distinguish between UI elements.
    pub id: String,
    /// X coordinate, 0 is the left edge, while the width of the screen is the right edge.
    pub x: f32,
    /// Y coordinate, 0 is the top edge, while the height of the screen is the bottom edge.
    pub y: f32,
    /// Z order, entities with a lower Z order will be rendered on top of entities with a higher
    /// Z order.
    pub z: f32,
    /// The width of this UI element
    pub width: f32,
    /// The height of this UI element
    pub height: f32,
    /// This function is called with this element and the new size whenever the window is resized.
    ///
    /// Do not try and replace this while the inner function  is being called.  Whatever you put
    /// here would be overwritten.
    pub resize_fn: Option<Box<FnMut(&mut UiTransform, (f32, f32)) + Send + Sync>>,
}

impl Component for UiTransform {
    type Storage = DenseVecStorage<Self>;
}

/// This system rearranges UI elements whenever the screen is resized using their resize_fn.
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
        Fetch<'a, EventChannel<Event>>,
    );

    fn run(&mut self, (mut transform, events): Self::SystemData) {
        use std::mem::replace;

        for event in events
            .lossy_read(&mut self.event_reader)
            .expect("ResizeSystem failed!")
        {
            if let &Event::WindowEvent {
                event: WindowEvent::Resized(width, height),
                ..
            } = event
            {
                for mut transform in (&mut transform).join() {
                    if transform.resize_fn.is_some() {
                        let mut resize_fn = replace(&mut transform.resize_fn, None);
                        (resize_fn.as_mut().unwrap())(
                            &mut transform,
                            (width as f32, height as f32),
                        );
                        transform.resize_fn = resize_fn;
                    }
                }
            }
        }
    }
}
