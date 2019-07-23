//! Input system

use crate::{BindingTypes, Bindings, InputEvent, InputHandler};
use amethyst_core::{
    ecs::prelude::{Read, ReadExpect, System, World, Write},
    shrev::{EventChannel, ReaderId},
};
use amethyst_window::ScreenDimensions;
use winit::Event;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// Input system
///
/// Will read `winit::Event` from `EventHandler<winit::Event>`, process them with `InputHandler`,
/// and push the results in `EventHandler<InputEvent>`.
#[derive(Debug)]
pub struct InputSystem<T: BindingTypes> {
    reader: ReaderId<Event>,
    bindings: Option<Bindings<T>>,
}

impl<T: BindingTypes> InputSystem<T> {
    /// Create a new input system. Needs a reader id for `EventHandler<winit::Event>`.
    pub fn new(mut world: &mut World, bindings: Option<Bindings<T>>) -> Self {
        use amethyst_core::ecs::prelude::SystemData;
        <Self as System<'_>>::SystemData::setup(&mut world);
        let reader = world.fetch_mut::<EventChannel<Event>>().register_reader();
        if let Some(ref bindings) = bindings {
            world.fetch_mut::<InputHandler<T>>().bindings = bindings.clone();
        }
        InputSystem { reader, bindings }
    }

    fn process_event(
        event: &Event,
        handler: &mut InputHandler<T>,
        output: &mut EventChannel<InputEvent<T>>,
        hidpi: f32,
    ) {
        handler.send_event(event, output, hidpi as f32);
    }
}

impl<'a, T: BindingTypes> System<'a> for InputSystem<T> {
    type SystemData = (
        Read<'a, EventChannel<Event>>,
        Write<'a, InputHandler<T>>,
        Write<'a, EventChannel<InputEvent<T>>>,
        ReadExpect<'a, ScreenDimensions>,
    );

    fn run(&mut self, (input, mut handler, mut output, screen_dimensions): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("input_system");

        handler.send_frame_begin();
        for event in input.read(&mut self.reader) {
            Self::process_event(
                event,
                &mut *handler,
                &mut *output,
                screen_dimensions.hidpi_factor() as f32,
            );
        }
    }
}
