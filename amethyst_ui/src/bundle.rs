//! ECS rendering bundle

use amethyst_core::bundle::{ECSBundle, Result};
use specs::{DispatcherBuilder, World};

use super::*;

/// Rendering bundle
///
/// Will register all necessary components needed for rendering, along with any resources.
/// Will also register asset contexts with the asset `Loader`, and add systems for merging
/// `AssetFuture` into its related component.
///
pub struct UiBundle;

impl UiBundle {
    /// Create a new render bundle
    pub fn new() -> Self {
        UiBundle
    }
}

impl<'a, 'b> ECSBundle<'a, 'b> for UiBundle {
    fn build(
        self,
        world: &mut World,
        builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        world.register::<UiImage>();
        world.register::<UiTransform>();

        Ok(builder)
    }
}
