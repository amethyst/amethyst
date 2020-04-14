//! Input system
use derive_new::new;
use winit::Event;

use std::{collections::HashMap, marker::PhantomData};

use crate::{BindingTypes, Bindings, Context, InputEvent, InputHandler};
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
pub struct InputSystemDesc<C, T>
where
    C: Context,
    T: BindingTypes,
{
    bindings: Option<HashMap<C, Bindings<T>>>,
}

impl<'a, 'b, C, T> SystemDesc<'a, 'b, InputSystem<C, T>> for InputSystemDesc<C, T>
where
    C: Context,
    T: BindingTypes,
{
    fn build(self, world: &mut World) -> InputSystem<C, T> {
        <InputSystem<C, T> as System<'_>>::SystemData::setup(world);

        let reader = world.fetch_mut::<EventChannel<Event>>().register_reader();
        if let Some(bindings) = self.bindings {
            let mut handler = world.fetch_mut::<InputHandler<C, T>>();
            for (context, bindings) in bindings {
                handler.set_bindings_for_context(context, bindings);
            }
        }

        InputSystem::new(reader)
    }
}

/// Input system
///
/// Will read `winit::Event` from `EventHandler<winit::Event>`, process them with `InputHandler`,
/// and push the results in `EventHandler<InputEvent>`.
#[derive(Debug)]
pub struct InputSystem<C, T>
where
    T: BindingTypes,
{
    reader: ReaderId<Event>,
    phantom: PhantomData<(C, T)>,
}

impl<C: Context, T: BindingTypes> InputSystem<C, T> {
    /// Create a new input system. Needs a reader id for `EventHandler<winit::Event>`.
    pub fn new(reader: ReaderId<Event>) -> Self {
        InputSystem {
            reader,
            phantom: PhantomData,
        }
    }

    fn process_event(
        event: &Event,
        handler: &mut InputHandler<C, T>,
        output: &mut EventChannel<InputEvent<T>>,
        hidpi: f32,
    ) {
        handler.send_event(event, output, hidpi as f32);
    }
}

impl<'a, C: Context, T: BindingTypes> System<'a> for InputSystem<C, T> {
    type SystemData = (
        Read<'a, EventChannel<Event>>,
        Write<'a, InputHandler<C, T>>,
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
