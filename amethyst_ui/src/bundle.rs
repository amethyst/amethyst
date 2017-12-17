//! ECS rendering bundle

use amethyst_assets::{AssetStorage, Handle, Processor};
use amethyst_core::bundle::{ECSBundle, Result};
use shrev::EventChannel;
use specs::{DispatcherBuilder, World};
use winit::Event;

use super::*;

/// UI bundle
///
/// Will register all necessary components and systems needed for UI, along with any resources.
pub struct UiBundle;

impl UiBundle {
    /// Create a new UI bundle
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
        world.register::<UiText>();
        world.register::<TextEditing>();
        world.register::<UiResize>();
        world.register::<Handle<FontAsset>>();
        world.add_resource(AssetStorage::<FontAsset>::new());
        world.add_resource(UiFocused { entity: None });
        let reader_1 = world
            .write_resource::<EventChannel<Event>>()
            .register_reader();
        let reader_2 = world
            .write_resource::<EventChannel<Event>>()
            .register_reader();
        Ok(
            builder
                .add(Processor::<FontAsset>::new(), "font_processor", &[])
                .add(UiSystem::new(reader_1), "ui_system", &["font_processor"])
                .add(ResizeSystem::new(reader_2), "ui_resize_system", &[]),
        )
    }
}
