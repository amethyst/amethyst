//! Input system

use std::hash::Hash;
use std::marker;

use shrev::{EventChannel, ReaderId};
use specs::{Fetch, FetchMut, System};
use winit::Event;

use {InputEvent, InputHandler};

/// Input system
///
/// Will read `winit::Event` from `EventHandler<winit::Event>`, process them with `InputHandler`,
/// and push the results in `EventHandler<InputEvent>`.
pub struct InputSystem<AX, AC> {
    m: marker::PhantomData<(AX, AC)>,
    reader: ReaderId,
}

impl<AX, AC> InputSystem<AX, AC> {
    /// Create a new input system. Needs a reader id for `EventHandler<winit::Event>`.
    pub fn new(reader: ReaderId) -> Self {
        Self {
            m: marker::PhantomData,
            reader,
        }
    }

    fn process_event(
        event: &Event,
        handler: &mut InputHandler<AX, AC>,
        output: &mut EventChannel<InputEvent<AC>>,
    ) where
        AX: Hash + Eq + Clone + Send + Sync + 'static,
        AC: Hash + Eq + Clone + Send + Sync + 'static,
    {
        if let &Event::WindowEvent { ref event, .. } = event {
            handler.send_event(event, output);
        }
    }
}

impl<'a, AX, AC> System<'a> for InputSystem<AX, AC>
where
    AX: Hash + Eq + Clone + Send + Sync + 'static,
    AC: Hash + Eq + Clone + Send + Sync + 'static,
{
    type SystemData = (
        Fetch<'a, EventChannel<Event>>,
        FetchMut<'a, InputHandler<AX, AC>>,
        FetchMut<'a, EventChannel<InputEvent<AC>>>,
    );

    fn run(&mut self, (input, mut handler, mut output): Self::SystemData) {
        match input.lossy_read(&mut self.reader) {
            Ok(data) => for d in data {
                Self::process_event(d, &mut *handler, &mut *output);
            },
            _ => (),
        }
    }
}
