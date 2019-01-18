//! ECS rendering bundle

use std::{hash::Hash, marker::PhantomData};

use derive_new::new;

use amethyst_assets::Processor;
use amethyst_audio::AudioFormat;
use amethyst_core::{
    bundle::{Result, SystemBundle},
    specs::prelude::DispatcherBuilder,
};
use amethyst_renderer::{BlinkSystem, TextureFormat};

use crate::{
    CacheSelectionOrderSystem, FontAsset, FontFormat, NoCustomUi, ResizeSystem,
    SelectionKeyboardSystem, SelectionMouseSystem, TextEditingInputSystem, TextEditingMouseSystem,
    ToNativeWidget, UiButtonActionRetriggerSystem, UiButtonSystem, UiLoaderSystem, UiMouseSystem,
    UiSoundRetriggerSystem, UiSoundSystem, UiTransformSystem,
};

/// UI bundle
///
/// Will register all necessary components and systems needed for UI, along with any resources.
/// The generic types A and B represent the A and B generic parameter of the InputHandler<A,B>.
///
/// Will fail with error 'No resource with the given id' if the InputBundle is not added.
#[derive(new)]
pub struct UiBundle<A = String, B = String, C = NoCustomUi, G = ()> {
    #[new(default)]
    _marker: PhantomData<(A, B, C, G)>,
}

impl<'a, 'b, A, B, C, G> SystemBundle<'a, 'b> for UiBundle<A, B, C, G>
where
    A: Send + Sync + Eq + Hash + Clone + 'static,
    B: Send + Sync + Eq + Hash + Clone + 'static,
    C: ToNativeWidget,
    G: Send + Sync + PartialEq + 'static,
{
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
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
            UiButtonSystem::new(),
            "ui_button_system",
            &["ui_mouse_system"],
        );

        builder.add(
            UiButtonActionRetriggerSystem::new(),
            "ui_button_action_retrigger_system",
            &["ui_button_system"],
        );
        builder.add(UiSoundSystem::new(), "ui_sound_system", &[]);
        builder.add(
            UiSoundRetriggerSystem::new(),
            "ui_sound_retrigger_system",
            &["ui_sound_system"],
        );

        // Required for text editing. You want the cursor image to blink.
        builder.add(BlinkSystem, "blink_system", &[]);

        Ok(())
    }
}
