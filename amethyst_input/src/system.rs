//! Input system

use std::hash::Hash;

use winit::Event;

use amethyst_core::{
    shrev::{EventChannel, ReaderId},
    specs::prelude::{Read, Resources, System, Write},
};

use {Bindings, InputEvent, InputHandler};

/// Input system
///
/// Will read `winit::Event` from `EventHandler<winit::Event>`, process them with `InputHandler`,
/// and push the results in `EventHandler<InputEvent>`.
pub struct InputSystem<AX, AC>
where
    AX: Hash + Eq,
    AC: Hash + Eq,
{
    reader: Option<ReaderId<Event>>,
    bindings: Option<Bindings<AX, AC>>,
}

impl<AX, AC> InputSystem<AX, AC>
where
    AX: Hash + Eq,
    AC: Hash + Eq,
{
    /// Create a new input system. Needs a reader id for `EventHandler<winit::Event>`.
    pub fn new(bindings: Option<Bindings<AX, AC>>) -> Self {
        InputSystem {
            reader: None,
            bindings,
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
        handler.send_event(event, output);
    }
}

impl<'a, AX, AC> System<'a> for InputSystem<AX, AC>
where
    AX: Hash + Eq + Clone + Send + Sync + 'static,
    AC: Hash + Eq + Clone + Send + Sync + 'static,
{
    type SystemData = (
        Read<'a, EventChannel<Event>>,
        Write<'a, InputHandler<AX, AC>>,
        Write<'a, EventChannel<InputEvent<AC>>>,
    );

    fn run(&mut self, (input, mut handler, mut output): Self::SystemData) {
        for event in input.read(
            &mut self
                .reader
                .as_mut()
                .expect("`InputSystem::setup` was not called before `InputSystem::run`"),
        ) {
            Self::process_event(event, &mut *handler, &mut *output);
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::specs::prelude::SystemData;
        Self::SystemData::setup(res);
        self.reader = Some(res.fetch_mut::<EventChannel<Event>>().register_reader());
        if let Some(ref bindings) = self.bindings {
            res.fetch_mut::<InputHandler<AX, AC>>().bindings = bindings.clone();
        }
    }
}
