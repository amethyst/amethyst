//! ECS rendering bundle

use std::marker::PhantomData;

use amethyst_core::{
    ecs::*,
};
use amethyst_error::Error;
use derive_new::new;
use crate::{WidgetId, build_ui_transform_system, build_ui_mouse_system, FontAsset, build_cache_selection_system, build_selection_mouse_system, build_selection_keyboard_system, build_text_editing_mouse_system, build_text_editing_input_system, build_resize_system, build_ui_button_system, build_drag_widget_system, build_button_action_retrigger_system, build_ui_sound_system, build_ui_sound_retrigger_system, build_blink_system};
use amethyst_rendy::build_texture_processor;
use amethyst_assets::build_asset_processor_system;

/// UI bundle
///
/// Will register all necessary components and systems needed for UI, along with any resources.
/// The generic type T represent the T generic parameter of the InputHandler<T>.
///
/// Will fail with error 'No resource with the given id' if either the InputBundle or TransformBundle are not added.
#[derive(new, Debug)]
pub struct UiBundle</*C = NoCustomUi, */W = u32, G = ()> {
    #[new(default)]
    _marker: PhantomData<(/*C,*/ W, G)>,
}

impl</*C,*/ W, G> SystemBundle for UiBundle</*C,*/ W, G>
where
    //C: ToNativeWidget,
    W: WidgetId,
    G: Send + Sync + PartialEq + 'static,
{
    fn load(&mut self, _world: &mut World, resources: &mut Resources, builder: &mut DispatcherBuilder) -> Result<(), Error> {

        builder
            .add_system(build_ui_transform_system(resources))
            .add_system(build_ui_mouse_system(resources))
            .add_system(build_asset_processor_system::<FontAsset>())
            .add_system(build_cache_selection_system::<G>( resources))
            .add_system(build_selection_mouse_system::<G>( resources))
            .add_system(build_selection_keyboard_system::<G>( resources))
            .add_system(build_text_editing_mouse_system(resources))
            .add_system(build_text_editing_input_system(resources))
            .add_system(build_resize_system(resources))
            .add_system(build_ui_button_system(resources))
            .add_system(build_drag_widget_system(resources))
            .add_system(build_button_action_retrigger_system(resources))
            .add_system(build_ui_sound_system(resources))
            .add_system(build_ui_sound_retrigger_system(resources))
            .add_system(build_blink_system());
        /*
                builder.add_system(
                    UiLoaderSystemDesc::<<C as ToNativeWidget>::PrefabData, W>::default().build(world),
                );
        */
        Ok(())
    }

    fn unload(&mut self, _world: &mut World,  _resources: &mut Resources) -> Result<(), Error> {
        unimplemented!()
    }
}
