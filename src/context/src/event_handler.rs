extern crate glutin;
pub use glutin::{Event, ElementState, ScanCode,
                 VirtualKeyCode, MouseScrollDelta,
                 TouchPhase, MouseButton, Touch};

use std::collections::vec_deque::{VecDeque, Drain};
use video_context::VideoContext;

pub type EventIter<'a> = Drain<'a, Event>;

pub struct EventHandler {
    queue: VecDeque<Event>,
}

impl EventHandler {
    pub fn new() -> EventHandler {
        EventHandler {
            queue: VecDeque::<Event>::new()
        }
    }

    pub fn poll(&mut self) -> EventIter {
        self.queue.drain(..)
    }

    pub fn publish(&mut self, event: Event) {
        self.queue.push_back(event);
    }
}

pub fn populate_event_handler(video_context: &mut VideoContext) -> EventHandler {
    let mut event_handler = EventHandler::new();
    match *video_context {
        VideoContext::OpenGL { ref window, .. } =>
            for event in window.poll_events() {
                event_handler.publish(event);
            },
        #[cfg(windows)]
        VideoContext::Direct3D {  } =>
            // stub
            event_handler.publish(Event::Closed),
    }
    event_handler
}
