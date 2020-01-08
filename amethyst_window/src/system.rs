use crate::ScreenDimensions;
use amethyst_core::ecs::{ReadExpect, System, WriteExpect};
use winit::{
    dpi::{LogicalSize, Size},
    window::Window,
};

/// System for opening and managing the window.
#[derive(Debug)]
pub struct WindowSystem;

impl WindowSystem {
    /// Create a new `WindowSystem` wrapping the provided `Window`
    pub fn new() -> Self {
        Self
    }

    fn manage_dimensions(&mut self, mut screen_dimensions: &mut ScreenDimensions, window: &Window) {
        let width = screen_dimensions.w;
        let height = screen_dimensions.h;

        // Send resource size changes to the window
        if screen_dimensions.dirty {
            window.set_inner_size(Size::Logical(LogicalSize::from((width, height))));
            screen_dimensions.dirty = false;
        }

        let hidpi = window.scale_factor();

        let size = window.inner_size();
        let (window_width, window_height): (f64, f64) = size.to_logical::<f64>(hidpi).into();

        // Send window size changes to the resource
        if (window_width, window_height) != (width, height) {
            screen_dimensions.update(window_width, window_height);

            // We don't need to send the updated size of the window back to the window itself,
            // so set dirty to false.
            screen_dimensions.dirty = false;
        }
        screen_dimensions.update_hidpi_factor(hidpi);
    }
}

impl<'a> System<'a> for WindowSystem {
    type SystemData = (WriteExpect<'a, ScreenDimensions>, ReadExpect<'a, Window>);

    fn run(&mut self, (mut screen_dimensions, window): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("window_system");

        self.manage_dimensions(&mut screen_dimensions, &window);
    }
}
