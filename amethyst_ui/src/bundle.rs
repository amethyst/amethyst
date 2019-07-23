//! ECS rendering bundle

use crate::{
    BlinkSystem, CacheSelectionOrderSystem, FontAsset, NoCustomUi, ResizeSystem,
    SelectionKeyboardSystem, SelectionMouseSystem, TextEditingInputSystem, TextEditingMouseSystem,
    ToNativeWidget, UiButtonActionRetriggerSystem, UiButtonSystem, UiLoaderSystem, UiMouseSystem,
    UiSoundRetriggerSystem, UiSoundSystem, UiTransformSystem, WidgetId,
};
use amethyst_assets::Processor;
use amethyst_core::{bundle::SystemBundle, ecs::prelude::DispatcherBuilder};
use amethyst_error::Error;
use amethyst_input::BindingTypes;
use derive_new::new;
use std::marker::PhantomData;

/// UI bundle
///
/// Will register all necessary components and systems needed for UI, along with any resources.
/// The generic type T represent the T generic parameter of the InputHandler<T>.
///
/// Will fail with error 'No resource with the given id' if the InputBundle is not added.
#[derive(new, Debug)]
pub struct UiBundle<T: BindingTypes, C = NoCustomUi, W = u32, G = ()> {
    #[new(default)]
    _marker: PhantomData<(T, C, W, G)>,
}

impl<'a, 'b, T, C, W, G> SystemBundle<'a, 'b> for UiBundle<T, C, W, G>
where
    T: BindingTypes,
    C: ToNativeWidget,
    W: WidgetId,
    G: Send + Sync + PartialEq + 'static,
{
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(
            UiLoaderSystem::<<C as ToNativeWidget>::PrefabData, W>::default(),
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
            SelectionMouseSystem::<G, T>::new(),
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
            UiMouseSystem::<T>::new(),
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
