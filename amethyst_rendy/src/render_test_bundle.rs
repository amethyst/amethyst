use std::marker::PhantomData;

use amethyst_core::{bundle::SystemBundle, ecs::DispatcherBuilder};
use amethyst_error::Error;
use derive_new::new;

use crate::{types::Backend, PluggableRenderingBundle};

/// Adds basic rendering system to the dispatcher.
///
/// This test bundle requires the user to also add the `TransformBundle`.
///
/// This is only meant for testing and only provides very basic sprite rendering. You need to enable
/// the `test-support` flag to use this.
#[derive(Debug, new)]
pub struct RenderTestBundle<B>(PhantomData<B>);

impl<'a, 'b, B> SystemBundle<'a, 'b> for RenderTestBundle<B>
where
    B: Backend,
{
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        let mut bundle = PluggableRenderingBundle::<B>::new()
            .with_plugin(crate::plugins::RenderFlat2D::default());

        #[cfg(feature = "window")]
        bundle.add_plugin(crate::plugins::RenderToWindow::from_config(
            amethyst_window::DisplayConfig {
                visibility: false,
                ..Default::default()
            },
        ));

        bundle.build(builder)?;
        Ok(())
    }
}

/// Add basic rendering system to the dispatcher.
///
/// This is only meant for automated testing and doesn't render anything. You need to enable the
///// `test-support` flag to use this.
#[derive(Debug, new)]
pub struct RenderEmptyBundle<B>(PhantomData<B>);

impl<'a, 'b, B> SystemBundle<'a, 'b> for RenderEmptyBundle<B>
where
    B: Backend,
{
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        let bundle = PluggableRenderingBundle::<B>::new();
        bundle.build(builder)?;
        Ok(())
    }
}
