use crate::WindowSystem;
use amethyst_core::{bundle::SystemBundle, ecs::World, shred::DispatcherBuilder};
use amethyst_error::Error;

/// Screen width used in predefined display configuration.
#[cfg(feature = "test-support")]
pub const SCREEN_WIDTH: u32 = 800;
/// Screen height used in predefined display configuration.
#[cfg(feature = "test-support")]
pub const SCREEN_HEIGHT: u32 = 600;

/// Bundle providing easy initializing of the `WindowSystem` used for managing WindowDimensions
#[derive(Debug)]
pub struct WindowBundle {}

impl WindowBundle {
    /// Creates new WindowBundle
    pub fn new() -> Self {
        Self {}
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for WindowBundle {
    #[cfg(not(feature = "wasm"))]
    fn build(self, _: &mut World, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add_thread_local(WindowSystem::new());
        Ok(())
    }

    #[cfg(feature = "wasm")]
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add_thread_local(WindowSystem::new());

        use crate::WindowRes;
        use amethyst_core::{ecs::ReadExpect, shred::SystemData};
        use winit::platform::web::WindowExtWebSys;

        let window = <ReadExpect<'_, WindowRes>>::fetch(world);
        let window = &**window;
        let canvas = window.canvas();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        body.append_child(&canvas)
            .expect("Append canvas to HTML body");

        Ok(())
    }
}
