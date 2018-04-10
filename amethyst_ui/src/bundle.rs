//! ECS rendering bundle

use amethyst_assets::{AssetStorage, Handle, Processor};
use amethyst_core::Parent;
use amethyst_core::bundle::{ECSBundle, Result};
use amethyst_core::specs::{DispatcherBuilder, World};
use shrev::EventChannel;
use std::hash::Hash;
use std::marker::PhantomData;
use winit::Event;

use super::*;

/// UI bundle
///
/// Will register all necessary components and systems needed for UI, along with any resources.
/// The generic types A and B represent the A and B generic parameter of the InputHandler<A,B>.
///
/// Will fail with error 'No resource with the given id' if the InputBundle is not added.
pub struct UiBundle<A, B> {
    _marker1: PhantomData<A>,
    _marker2: PhantomData<B>,
}

impl<A, B> UiBundle<A, B> {
    /// Create a new UI bundle
    pub fn new() -> Self {
        UiBundle {
            _marker1: PhantomData,
            _marker2: PhantomData,
        }
    }
}

impl<'a, 'b, A, B> ECSBundle<'a, 'b> for UiBundle<A, B>
where
    A: Send + Sync + Eq + Hash + Clone + 'static,
    B: Send + Sync + Eq + Hash + Clone + 'static,
{
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
        world.register::<MouseReactive>();
        world.register::<Anchored>();
        world.register::<Stretched>();
        world.register::<Parent>();

        world.add_resource(AssetStorage::<FontAsset>::new());
        world.add_resource(UiFocused { entity: None });
        world.add_resource(EventChannel::<UiEvent>::new());

        let reader_1 = world
            .write_resource::<EventChannel<Event>>()
            .register_reader();
        let reader_2 = world
            .write_resource::<EventChannel<Event>>()
            .register_reader();

        Ok(builder
            .add(Processor::<FontAsset>::new(), "font_processor", &[])
            .add(UiSystem::new(reader_1), "ui_system", &["font_processor"])
            .add(ResizeSystem::new(reader_2), "ui_resize_system", &[])
            .add(UiMouseSystem::<A, B>::new(), "ui_mouse_system", &[])
            .add(UiLayoutSystem::new(), "ui_layout", &["ui_system"])
            .add(UiParentSystem::new(), "ui_parent", &["ui_layout"]))
    }
}
