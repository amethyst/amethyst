//! ECS rendering bundle

use std::{hash::Hash, marker::PhantomData};

use derive_new::new;

use amethyst_assets::{Processor, Format};
use amethyst_audio::AudioFormat;
use amethyst_core::{bundle::SystemBundle, ecs::prelude::DispatcherBuilder};
use amethyst_error::Error;

use crate::{
    CacheSelectionOrderSystem, FontAsset, FontFormat, NoCustomUi, ResizeSystem,
    SelectionKeyboardSystem, SelectionMouseSystem, TextEditingInputSystem, TextEditingMouseSystem,
    ToNativeWidget, UiButtonActionRetriggerSystem, UiButtonSystem, UiLoaderSystem, UiMouseSystem,
    UiSoundRetriggerSystem, UiSoundSystem, UiTransformSystem, WidgetId,
    render::{UiRenderer, TexturePrefab},
};

/// UI bundle
///
/// Will register all necessary components and systems needed for UI, along with any resources.
/// The generic types A and B represent the A and B generic parameter of the InputHandler<A,B>.
///
/// Will fail with error 'No resource with the given id' if the InputBundle is not added.
#[derive(new)]
pub struct UiBundle<R, I, T, A = String, B = String, C = NoCustomUi, W = u32, G = ()> {
    #[new(default)]
    _marker: PhantomData<(R, I, T, A, B, C, W, G)>,
}

impl<'a, 'b, R, I, T, A, B, C, W, G> SystemBundle<'a, 'b> for UiBundle<R, I, T, A, B, C, W, G>
where
    A: Send + Sync + Eq + Hash + Clone + 'static,
    B: Send + Sync + Eq + Hash + Clone + 'static,
    C: ToNativeWidget<R, I, T>,
    W: WidgetId,
    G: Send + Sync + PartialEq + 'static,
    R: UiRenderer,
    I: Format<R::Texture, Options = ()>,
    T: TexturePrefab<R, I>,
{
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(
            UiLoaderSystem::<
                R,
                I,
                T,
                AudioFormat,
                FontFormat,
                <C as ToNativeWidget>::PrefabData,
                W,
            >::default(),
            "ui_loader",
            &[],
        );
        builder.add(
            UiTransformSystem::default(),
            "ui_transform",
            &["transform_system"],
        );
        builder.add(
            Processor::<FontAsset>::new(),
            "font_processor",
            &["ui_loader"],
        );
        builder.add(
            CacheSelectionOrderSystem::<G>::new(),
            "selection_order_cache",
            &[],
        );
        builder.add(
            SelectionMouseSystem::<G, A, B>::new(),
            "ui_mouse_selection",
            &[],
        );
        builder.add(
            SelectionKeyboardSystem::<G>::new(),
            "ui_keyboard_selection",
            // Because when you press tab, you want to override the previously selected elements.
            &["ui_mouse_selection"],
        );
        builder.add(
            TextEditingMouseSystem::new(),
            "ui_text_editing_mouse_system",
            &["ui_mouse_selection", "ui_keyboard_selection"],
        );
        builder.add(
            TextEditingInputSystem::new(),
            "ui_text_editing_input_system",
            // Hard requirement. The system assumes the text to edit is selected.
            &["ui_mouse_selection", "ui_keyboard_selection"],
        );
        builder.add(ResizeSystem::new(), "ui_resize_system", &[]);
        builder.add(
            UiMouseSystem::<A, B>::new(),
            "ui_mouse_system",
            &["ui_transform"],
        );
        builder.add(
            UiButtonSystem::<R>::new(),
            "ui_button_system",
            &["ui_mouse_system"],
        );

        builder.add(
            UiButtonActionRetriggerSystem::<R>::new(),
            "ui_button_action_retrigger_system",
            &["ui_button_system"],
        );
        builder.add(UiSoundSystem::new(), "ui_sound_system", &[]);
        builder.add(
            UiSoundRetriggerSystem::new(),
            "ui_sound_retrigger_system",
            &["ui_sound_system"],
        );

        // TODO(happens): Move this into ui. Why is this even here?
        // Required for text editing. You want the cursor image to blink.
        // builder.add(BlinkSystem, "blink_system", &[]);

        Ok(())
    }
}
