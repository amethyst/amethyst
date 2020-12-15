//! Input system
use amethyst_core::{
    ecs::*,
    shrev::{EventChannel, ReaderId},
};
use amethyst_window::ScreenDimensions;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;
use winit::Event;

use crate::{InputEvent, InputHandler};

/// Will read `winit::Event` from `EventHandler<winit::Event>`, process them with `InputHandler`,
/// and push the results in `EventHandler<InputEvent>`.
#[derive(Debug)]
pub struct InputSystem {
    // reads input events from winit
    pub(crate) reader: ReaderId<Event>,
}

impl System<'static> for InputSystem {
    fn build(&'static mut self) -> Box<dyn systems::ParallelRunnable> {
        Box::new(
            SystemBuilder::new("InputSystem")
                .read_resource::<EventChannel<Event>>()
                .write_resource::<InputHandler>()
                .write_resource::<EventChannel<InputEvent>>()
                .read_resource::<ScreenDimensions>()
                .build(
                    move |_commands,
                          _world,
                          (input, handler, output, screen_dimensions),
                          _query| {
                        #[cfg(feature = "profiler")]
                        profile_scope!("input_system");

                        handler.send_frame_begin();
                        for event in input.read(&mut self.reader) {
                            handler.send_event(
                                event,
                                output,
                                screen_dimensions.hidpi_factor() as f32,
                            );
                        }
                    },
                ),
        )
    }
}
