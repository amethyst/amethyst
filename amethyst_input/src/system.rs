//! Input system

use crate::{BindingTypes, Bindings, InputEvent, InputHandler};
use amethyst_core::{
    ecs::prelude::{Read, ReadExpect, Resources, System, Write},
    shrev::{EventChannel, ReaderId},
};
use amethyst_window::ScreenDimensions;
use winit::Event;

/// Input system
///
/// Will read `winit::Event` from `EventHandler<winit::Event>`, process them with `InputHandler`,
/// and push the results in `EventHandler<InputEvent>`.
pub struct InputSystem<T: BindingTypes> {
    reader: Option<ReaderId<Event>>,
    bindings: Option<Bindings<T>>,
}

impl<T: BindingTypes> InputSystem<T> {
    /// Create a new input system. Needs a reader id for `EventHandler<winit::Event>`.
    pub fn new(bindings: Option<Bindings<T>>) -> Self {
        InputSystem {
            reader: None,
            bindings,
        }
    }

    fn process_event(
        event: &Event,
        handler: &mut InputHandler<T>,
        output: &mut EventChannel<InputEvent<T::Action>>,
        hidpi: f64,
    ) {
        handler.send_event(event, output, hidpi);
    }
}

impl<'a, T: BindingTypes> System<'a> for InputSystem<T> {
    type SystemData = (
        Read<'a, EventChannel<Event>>,
        Write<'a, InputHandler<T>>,
        Write<'a, EventChannel<InputEvent<T::Action>>>,
        ReadExpect<'a, ScreenDimensions>,
    );

    fn run(&mut self, (input, mut handler, mut output, screen_dimensions): Self::SystemData) {
        handler.send_frame_begin();
        for event in input.read(
            &mut self
                .reader
                .as_mut()
                .expect("`InputSystem::setup` was not called before `InputSystem::run`"),
        ) {
            Self::process_event(
                event,
                &mut *handler,
                &mut *output,
                screen_dimensions.hidpi_factor(),
            );
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::ecs::prelude::SystemData;
        Self::SystemData::setup(res);
        self.reader = Some(res.fetch_mut::<EventChannel<Event>>().register_reader());
        if let Some(ref bindings) = self.bindings {
            res.fetch_mut::<InputHandler<T>>().bindings = bindings.clone();
        }
    }
}
