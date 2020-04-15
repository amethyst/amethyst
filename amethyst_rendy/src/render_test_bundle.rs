use amethyst_core::{
    bundle::SystemBundle,
    ecs::{DispatcherBuilder, World},
};
use amethyst_error::Error;
use amethyst_window::EventLoop;

use crate::{types::Backend, RenderingBundle};

/// Adds basic rendering system to the dispatcher.
///
/// This test bundle requires the user to also add the `TransformBundle`.
///
/// This is only meant for testing and only provides very basic sprite rendering. You need to enable
/// the `test-support` flag to use this.
#[derive(Debug)]
pub struct RenderTestBundle<B>
where
    B: Backend,
{
    /// The rendering bundle to add to the dispatcher.
    rendering_bundle: RenderingBundle<B>,
}

impl<B> RenderTestBundle<B>
where
    B: Backend,
{
    /// Returns a new `RenderTestBundle`.
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let display_config = amethyst_window::DisplayConfig {
            visibility: false,
            ..Default::default()
        };
        let mut rendering_bundle = RenderingBundle::<B>::new(display_config.clone(), event_loop)
            .with_plugin(crate::plugins::RenderFlat2D::default());

        #[cfg(feature = "window")]
        rendering_bundle.add_plugin(crate::plugins::RenderToWindow::new());

        Self { rendering_bundle }
    }
}

impl<'a, 'b, B> SystemBundle<'a, 'b> for RenderTestBundle<B>
where
    B: Backend,
{
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        self.rendering_bundle.build(world, builder)
    }
}

/// Add basic rendering system to the dispatcher.
///
/// This is only meant for automated testing and doesn't render anything. You need to enable the
///// `test-support` flag to use this.
#[derive(Debug)]
pub struct RenderEmptyBundle<B>
where
    B: Backend,
{
    /// The rendering bundle to add to the dispatcher.
    rendering_bundle: RenderingBundle<B>,
}

impl<B> RenderEmptyBundle<B>
where
    B: Backend,
{
    /// Returns a new `RenderEmptyBundle`.
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let display_config = amethyst_window::DisplayConfig {
            visibility: false,
            ..Default::default()
        };
        let rendering_bundle = RenderingBundle::<B>::new(display_config, event_loop);

        Self { rendering_bundle }
    }
}

impl<'a, 'b, B> SystemBundle<'a, 'b> for RenderEmptyBundle<B>
where
    B: Backend,
{
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        self.rendering_bundle.build(world, builder)
    }
}
