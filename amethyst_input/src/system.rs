//! Input system
use derive_new::new;
use winit::event::Event;

use crate::{BindingTypes, Bindings, InputEvent, InputHandler};
use amethyst_core::{
    ecs::{
        prelude::{Read, ReadExpect, System, World, Write},
        SystemData,
    },
    shrev::{EventChannel, ReaderId},
    SystemDesc,
};
use amethyst_window::ScreenDimensions;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// Builds an `InputSystem`.
#[derive(Debug, new)]
pub struct InputSystemDesc<T>
where
    T: BindingTypes,
{
    bindings: Option<Bindings<T>>,
}

impl<'a, 'b, T> SystemDesc<'a, 'b, InputSystem<T>> for InputSystemDesc<T>
where
    T: BindingTypes,
{
    fn build(self, world: &mut World) -> InputSystem<T> {
        <InputSystem<T> as System<'_>>::SystemData::setup(world);

        let reader = world
            .fetch_mut::<EventChannel<Event<()>>>()
            .register_reader();
        if let Some(bindings) = self.bindings.as_ref() {
            world.fetch_mut::<InputHandler<T>>().bindings = bindings.clone();
        }

        InputSystem::new(reader, self.bindings)
    }
}

/// Input system
///
/// Will read `winit::Event` from `EventHandler<winit::Event>`, process them with `InputHandler`,
/// and push the results in `EventHandler<InputEvent>`.
#[derive(Debug)]
pub struct InputSystem<T>
where
    T: BindingTypes,
{
    reader: ReaderId<Event<()>>,
    bindings: Option<Bindings<T>>,
}

impl<T: BindingTypes> InputSystem<T> {
    /// Create a new input system. Needs a reader id for `EventHandler<winit::Event>`.
    pub fn new(reader: ReaderId<Event<()>>, bindings: Option<Bindings<T>>) -> Self {
        InputSystem { reader, bindings }
    }

    fn process_event(
        event: &Event<()>,
        handler: &mut InputHandler<T>,
        output: &mut EventChannel<InputEvent<T>>,
        hidpi: f32,
    ) {
        handler.send_event(event, output, hidpi as f32);
    }
}

impl<'a, T: BindingTypes> System<'a> for InputSystem<T> {
    type SystemData = (
        Read<'a, EventChannel<Event<()>>>,
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
