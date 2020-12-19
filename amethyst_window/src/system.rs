use amethyst_core::{
    dispatcher::System,
    ecs::{systems::ParallelRunnable, SystemBuilder},
};
use winit::{dpi::Size, window::Window};

use crate::resources::ScreenDimensions;

/// Manages window dimensions
#[derive(Debug)]
pub struct WindowSystem;

/// Builds window system that updates `ScreenDimensions` resource from a provided `Window`.
impl System<'_> for WindowSystem {
    fn build(&mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("WindowSystem")
                .write_resource::<ScreenDimensions>()
                .read_resource::<Window>()
                .build(|_commands, _world, (screen_dimensions, window), _query| {
                    let width = screen_dimensions.w;
                    let height = screen_dimensions.h;

                    // Send resource size changes to the window
                    if screen_dimensions.dirty {
                        window.set_inner_size(Size::Logical((width, height).into()));
                        screen_dimensions.dirty = false;
                    }

                    if let size = window.inner_size() {
                        let (window_width, window_height): (f64, f64) = size.into();

                        // Send window size changes to the resource
                        if (window_width, window_height) != (width, height) {
                            screen_dimensions.update(window_width, window_height);

                            // We don't need to send the updated size of the window back to the window itself,
                            // so set dirty to false.
                            screen_dimensions.dirty = false;
                        }
                    }
                }),
        )
    }
}
