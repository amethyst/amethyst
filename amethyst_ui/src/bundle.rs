//! ECS rendering bundle

use std::marker::PhantomData;

use amethyst_assets::AssetProcessorSystemBundle;
use amethyst_audio::Source;
use amethyst_core::{ecs::*, shrev::EventChannel};
use amethyst_error::Error;
use derive_new::new;
use winit::Event;

use crate::{
    button::{ui_button_action_retrigger_event_system, UiButtonSystem},
    drag::DragWidgetSystem,
    event::UiMouseSystem,
    layout::UiTransformSystem,
    resize::ResizeSystem,
    selection::{SelectionKeyboardSystem, SelectionMouseSystem},
    selection_order_cache::CacheSelectionSystem,
    sound::{ui_sound_event_retrigger_system, UiSoundSystem},
    text::TextEditingMouseSystem,
    text_editing::TextEditingInputSystem,
    BlinkSystem, CachedSelectionOrderResource, FontAsset, UiButtonAction, UiEvent, UiLabel,
    UiPlaySoundAction, WidgetId, Widgets,
};

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
        resources.insert(CachedSelectionOrderResource::default());

        let ui_btn_reader = resources
            .get_mut::<EventChannel<UiButtonAction>>()
            .unwrap()
            .register_reader();

        let ui_btn_action_retrigger_reader = resources
            .get_mut::<EventChannel<UiEvent>>()
            .unwrap()
            .register_reader();

        let text_editing_mouse_reader = resources
            .get_mut::<EventChannel<Event>>()
            .unwrap()
            .register_reader();

        let selection_mouse_reader = resources
            .get_mut::<EventChannel<UiEvent>>()
            .unwrap()
            .register_reader();

        let selection_keyboard_reader = resources
            .get_mut::<EventChannel<Event>>()
            .unwrap()
            .register_reader();
        let text_editing_input_reader = resources
            .get_mut::<EventChannel<Event>>()
            .unwrap()
            .register_reader();
        let drag_widget_reader = resources
            .get_mut::<EventChannel<UiEvent>>()
            .unwrap()
            .register_reader();
        builder
            .add_system(Box::new(UiTransformSystem::new()))
            .add_system(Box::new(UiMouseSystem::new()))
            .add_bundle(AssetProcessorSystemBundle::<FontAsset>::default())
            .add_system(Box::new(UiButtonSystem::new(ui_btn_reader)))
            .add_system(Box::new(ui_button_action_retrigger_event_system(
                ui_btn_action_retrigger_reader,
            )))
            .add_system(Box::new(CacheSelectionSystem::<G>::new()))
            .add_system(Box::new(TextEditingMouseSystem::new(
                text_editing_mouse_reader,
            )))
            .add_system(Box::new(SelectionMouseSystem::<G>::new(
                selection_mouse_reader,
            )))
            .add_system(Box::new(SelectionKeyboardSystem::<G>::new(
                selection_keyboard_reader,
            )))
            .add_system(Box::new(TextEditingInputSystem::new(
                text_editing_input_reader,
            )))
            .add_system(Box::new(ResizeSystem::new()))
            .add_system(Box::new(DragWidgetSystem::new(drag_widget_reader)))
            .add_bundle(AssetProcessorSystemBundle::<Source>::default())
            .add_system(Box::new(BlinkSystem));

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
            .add_system(Box::new(UiSoundSystem::new(
                resources
                    .get_mut::<EventChannel<UiPlaySoundAction>>()
                    .unwrap()
                    .register_reader(),
            )))
            .add_system(Box::new(ui_sound_event_retrigger_system(
                resources
                    .get_mut::<EventChannel<UiEvent>>()
                    .unwrap()
                    .register_reader(),
            )));
        Ok(())
    }

    fn unload(&mut self, _world: &mut World, _resources: &mut Resources) -> Result<(), Error> {
        // FIXME: should get all resources and remove them
        Ok(())
    }
}
