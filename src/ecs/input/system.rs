//! Input system

use std::hash::Hash;
use std::marker;

use input::{InputEvent, InputHandler};
use shrev::{EventHandler, EventReadData, ReaderId};
use winit::Event;

use ecs::{Fetch, FetchMut, System};

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

    fn process_event(event: &Event, handler: &mut InputHandler<AX, AC>, output: &mut EventHandler<InputEvent<AC>>)
    where
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
        Fetch<'a, EventHandler<Event>>,
        FetchMut<'a, InputHandler<AX, AC>>,
        FetchMut<'a, EventHandler<InputEvent<AC>>>,
    );

    fn run(&mut self, (input, mut handler, mut output): Self::SystemData) {
        match input.read(&mut self.reader) {
            Ok(EventReadData::Data(data)) => for d in data {
                Self::process_event(d, &mut *handler, &mut *output);
            },
            Ok(EventReadData::Overflow(data, lost)) => {
                eprintln!("WARNING: Event buffer overflow! {} events were lost!", lost); 
                for d in data {
                    Self::process_event(d, &mut *handler, &mut *output);
                }
            },
            _ => (),
        }
    }
}


