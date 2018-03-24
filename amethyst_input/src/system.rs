//! Input system

use std::hash::Hash;
use std::marker;

use amethyst_core::specs::prelude::{Fetch, FetchMut, System};
use shrev::{EventChannel, ReaderId};
use winit::Event;

use {InputEvent, InputHandler};

/// Input system
///
/// Will read `winit::Event` from `EventHandler<winit::Event>`, process them with `InputHandler`,
/// and push the results in `EventHandler<InputEvent>`.
pub struct InputSystem<AX, AC> {
    m: marker::PhantomData<(AX, AC)>,
    reader: ReaderId<Event>,
}

impl<AX, AC> InputSystem<AX, AC> {
    /// Create a new input system. Needs a reader id for `EventHandler<winit::Event>`.
    pub fn new(reader: ReaderId<Event>) -> Self {
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
        for event in input.read(&mut self.reader) {
            Self::process_event(event, &mut *handler, &mut *output);
        }
    }
}
