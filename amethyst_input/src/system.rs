//! Input system
use winit::Event;

use crate::{BindingTypes, InputEvent, InputHandler, Bindings};
use amethyst_core::{
    ecs::prelude::*,
    shrev::{EventChannel, ReaderId},
};
use amethyst_window::ScreenDimensions;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// Input system
///
/// Will read `winit::Event` from `EventHandler<winit::Event>`, process them with `InputHandler`,
/// and push the results in `EventHandler<InputEvent>`.
pub fn build_input_system<T: BindingTypes>(
    bindings: Option<Bindings<T>>,
) -> Box<dyn FnOnce(&mut World, &mut Resources) -> Box<dyn Schedulable>> {
    Box::new(move |_world, resources| {
        let mut reader = resources
            .get_mut::<EventChannel<Event>>()
            .unwrap()
            .register_reader();

        let mut handler = InputHandler::<T>::new();
        if let Some(bindings) = bindings.as_ref() {
            handler.bindings = bindings.clone();
        }

        resources.insert(handler);
        resources.insert(EventChannel::<InputEvent<T>>::new());

        SystemBuilder::<()>::new("InputSystem")
            .read_resource::<EventChannel<Event>>()
            .write_resource::<InputHandler<T>>()
            .write_resource::<EventChannel<InputEvent<T>>>()
            .read_resource::<ScreenDimensions>()
            .build(
                move |_commands, _world, (input, handler, output, screen_dimensions), _query| {
                    #[cfg(feature = "profiler")]
                    profile_scope!("input_system");

                    handler.send_frame_begin();
                    for event in input.read(&mut reader) {
                        handler.send_event(event, output, screen_dimensions.hidpi_factor() as f32);
                    }
                },
            )
    })
}
