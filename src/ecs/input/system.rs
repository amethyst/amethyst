//! Input system

use std::hash::Hash;
use std::marker;

use input::{InputEvent, InputHandler};
use shrev::{EventHandler, ReaderId};
use winit::Event;

use ecs::{Fetch, FetchMut, System};

/// Input system
///
/// Will read `winit::Event` from `EventHandler<winit::Event>`, process them with `InputHandler`,
/// and push the results in `EventHandler<InputEvent>`.
pub struct InputSystem<T> {
    m: marker::PhantomData<T>,
    reader: ReaderId,
}

impl<T> InputSystem<T> {
    /// Create a new input system. Needs a reader id for `EventHandler<winit::Event>`.
    pub fn new(reader: ReaderId) -> Self {
        Self {
            m: marker::PhantomData,
            reader,
        }
    }
}

impl<'a, T> System<'a> for InputSystem<T>
where
    T: Hash + Eq + Clone + Send + Sync + 'static,
{
    type SystemData = (
        Fetch<'a, EventHandler<Event>>,
        FetchMut<'a, InputHandler<T>>,
        FetchMut<'a, EventHandler<InputEvent<T>>>,
    );

    fn run(&mut self, (input, mut handler, mut output): Self::SystemData) {
        match input.read(&mut self.reader) {
            Ok(data) => for d in data {
                if let &Event::WindowEvent { ref event, .. } = d {
                    handler.send_event(event, &mut *output);
                }
            },

            Err(_) => (),
        }
    }
}
