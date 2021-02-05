//! Input system
use amethyst_core::{
    ecs::*,
    shrev::{EventChannel, ReaderId},
};
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;
use winit::event::Event;

use crate::{InputEvent, InputHandler};

/// Will read `winit::Event` from `EventHandler<winit::Event>`, process them with `InputHandler`,
/// and push the results in `EventHandler<InputEvent>`.
#[derive(Debug)]
pub struct InputSystem {
    // reads input events from winit
    pub(crate) reader: ReaderId<Event<'static, ()>>,
}

impl System for InputSystem {
    fn build(mut self) -> Box<dyn systems::ParallelRunnable> {
        Box::new(
            SystemBuilder::new("InputSystem")
                .read_resource::<EventChannel<Event<'static, ()>>>()
                .write_resource::<InputHandler>()
                .write_resource::<EventChannel<InputEvent>>()
                .build(move |_commands, _world, (input, handler, output), _query| {
                    #[cfg(feature = "profiler")]
                    profile_scope!("input_system");

                    handler.send_frame_begin();
                    for event in input.read(&mut self.reader) {
                        handler.send_event(event, output);
                    }
                }),
        )
    }
}
