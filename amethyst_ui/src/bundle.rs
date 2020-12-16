//! ECS rendering bundle

use std::marker::PhantomData;

use amethyst_assets::{AssetProcessorSystemBundle};
use amethyst_audio::Source;
use amethyst_core::{ecs::*, shrev::EventChannel};
use amethyst_error::Error;
use derive_new::new;

use crate::{build_blink_system, build_button_action_retrigger_system, build_cache_selection_system, build_drag_widget_system, build_resize_system, build_selection_keyboard_system, build_selection_mouse_system, build_text_editing_input_system, build_text_editing_mouse_system, build_ui_button_system, build_ui_mouse_system, build_ui_sound_retrigger_system, build_ui_sound_system, build_ui_transform_system, FontAsset, UiButtonAction, UiPlaySoundAction, WidgetId, Widgets, UiLabel};

/// UI bundle
///
/// Will register all necessary components and systems needed for UI, along with any resources.
/// The generic type T represent the T generic parameter of the InputHandler<T>.
///
/// Will fail with error 'No resource with the given id' if either the InputBundle or TransformBundle are not added.
#[derive(new, Debug, Default)]
pub struct UiBundle</*C = NoCustomUi, */ W = u32, G = ()> {
    #[new(default)]
    _marker: PhantomData<(/*C,*/ W, G)>,
}

impl</*C,*/ W, G> SystemBundle for UiBundle</*C,*/ W, G>
where
    //C: ToNativeWidget,
    W: WidgetId,
    G: Send + Sync + PartialEq + 'static,
{
    fn load(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        resources.insert(EventChannel::<UiButtonAction>::new());
        resources.insert(Widgets::<UiLabel, W>::new());

        builder
            .add_system(build_ui_transform_system(resources))
            .add_system(build_ui_mouse_system(resources))
            .add_bundle(AssetProcessorSystemBundle::<FontAsset>::default())
            .add_system(build_ui_button_system(resources))
            .add_system(build_button_action_retrigger_system(resources))
            .add_system(build_cache_selection_system::<G>(resources))
            .add_system(build_text_editing_mouse_system(resources))
            .add_system(build_selection_mouse_system::<G>(resources))
            .add_system(build_selection_keyboard_system::<G>(resources))
            .add_system(build_text_editing_input_system(resources))
            .add_system(build_resize_system(resources))
            .add_system(build_drag_widget_system(resources))
            .add_bundle(AssetProcessorSystemBundle::<Source>::default())
            .add_system(build_blink_system())
            ;

        /*
                builder.add_system(
                    UiLoaderSystemDesc::<<C as ToNativeWidget>::PrefabData, W>::default().build(world),
                );
        */
        Ok(())
    }

    fn unload(&mut self, _world: &mut World, _resources: &mut Resources) -> Result<(), Error> {
        // FIXME: should get all resources and remove them
        Ok(())
    }
}

/// Audio UI bundle
///
/// Will register all necessary components and systems needed for UI, along with any resources.
/// The generic type T represent the T generic parameter of the InputHandler<T>.
///
/// Will fail if no Output added. Add it with `amethyst_audio::output::init_output`
#[derive(new, Debug, Default)]
pub struct AudioUiBundle;

impl SystemBundle for AudioUiBundle {
    fn load(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        resources.insert(EventChannel::<UiPlaySoundAction>::new());
        builder
            .add_system(build_ui_sound_system(resources))
            .add_system(build_ui_sound_retrigger_system(resources))
        ;
        Ok(())
    }

    fn unload(&mut self, _world: &mut World, _resources: &mut Resources) -> Result<(), Error> {
        // FIXME: should get all resources and remove them
        Ok(())
    }
}
