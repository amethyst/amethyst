//! ECS rendering bundle

use std::{hash::Hash, marker::PhantomData};

use amethyst_assets::Processor;
use amethyst_audio::AudioFormat;
use amethyst_core::{
    bundle::{Result, SystemBundle},
    SimpleDispatcherBuilder,
};
use amethyst_renderer::TextureFormat;

use super::*;

/// UI bundle
///
/// Will register all necessary components and systems needed for UI, along with any resources.
/// The generic types A and B represent the A and B generic parameter of the InputHandler<A,B>.
///
/// Will fail with error 'No resource with the given id' if the InputBundle is not added.
pub struct UiBundle<A, B, C = NoCustomUi> {
    _marker: PhantomData<(A, B, C)>,
}

impl<A, B, C> UiBundle<A, B, C> {
    /// Create a new UI bundle
    pub fn new() -> Self {
        UiBundle {
            _marker: PhantomData,
        }
    }
}

impl<'a, 'b, 'c, A, B, C, D> SystemBundle<'a, 'b, 'c, D> for UiBundle<A, B, C>
where
    A: Send + Sync + Eq + Hash + Clone + 'static,
    B: Send + Sync + Eq + Hash + Clone + 'static,
    C: ToNativeWidget,
    D: SimpleDispatcherBuilder<'a, 'b, 'c>,
{
    fn build(self, builder: &mut D) -> Result<()> {
        builder.add(
            UiLoaderSystem::<
                AudioFormat,
                TextureFormat,
                FontFormat,
                <C as ToNativeWidget>::PrefabData,
            >::default(),
            "ui_loader",
            &[],
        );
        builder.add(
            Processor::<FontAsset>::new(),
            "font_processor",
            &["ui_loader"],
        );
        builder.add(
            UiKeyboardSystem::new(),
            "ui_keyboard_system",
            &["font_processor"],
        );
        builder.add(ResizeSystem::new(), "ui_resize_system", &[]);
        builder.add(
            UiTransformSystem::default(),
            "ui_transform",
            &["transform_system"],
        );
        builder.add(
            UiMouseSystem::<A, B>::new(),
            "ui_mouse_system",
            &["ui_transform"],
        );
        builder.add(
            UiButtonSystem::new(),
            "ui_button_system",
            &["ui_mouse_system"],
        );
        Ok(())
    }
}
