//! ECS rendering bundle

use amethyst_assets::{AssetStorage, Handle, Processor};
use amethyst_core::bundle::{ECSBundle, Result};
use specs::{DispatcherBuilder, World};

use super::*;

/// UI bundle
///
/// Will register all necessary components and systems needed for UI, along with any resources.
///
/// UiTextRenderer is registered with name "ui_text".
pub struct UiBundle {
    deps: &'static [&'static str],
}

impl UiBundle {
    /// Create a new UI bundle, the dependencies given will be the dependencies for the
    /// UiTextRenderer system.
    pub fn new(deps: &'static [&'static str]) -> Self {
        UiBundle {
            deps
        }
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
        world.register::<UiText>();
        world.register::<Handle<FontFileAsset>>();
        world.add_resource(AssetStorage::<FontFileAsset>::new());
        Ok(builder
            .add(UiTextRenderer, "ui_text", self.deps)
            .add(Processor::<FontFileAsset>::new(), "font_processor", &[])
        )
    }
}
