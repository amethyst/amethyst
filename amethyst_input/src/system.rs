//! Input system
use winit::Event;

use crate::{BindingTypes, InputEvent, InputHandler};
use amethyst_core::{
    ecs::*,
    shrev::{EventChannel, ReaderId},
};
use amethyst_window::ScreenDimensions;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// Input system
///
/// Will read `winit::Event` from `EventHandler<winit::Event>`, process them with `InputHandler`,
/// and push the results in `EventHandler<InputEvent>`.
pub fn build_input_system<T: BindingTypes>(mut reader: ReaderId<Event>) -> impl Runnable {
    SystemBuilder::new("InputSystem")
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
}
