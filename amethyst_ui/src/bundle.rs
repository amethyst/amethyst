//! ECS rendering bundle

use std::marker::PhantomData;

use amethyst_assets::ProcessingQueue;
use amethyst_core::{ecs::*, shrev::EventChannel};
use amethyst_error::Error;
use amethyst_rendy::types::DefaultBackend;
use derive_new::new;
use winit::event::Event;

use crate::{
    button::{ui_button_action_retrigger_event_system, UiButtonSystem},
    drag::DragWidgetSystem,
    event::UiMouseSystem,
    glyphs::{GlyphTextureData, GlyphTextureProcessorSystem},
    layout::UiTransformSystem,
    resize::ResizeSystem,
    selection::{SelectionKeyboardSystem, SelectionMouseSystem},
    selection_order_cache::CacheSelectionSystem,
    sound::{ui_sound_event_retrigger_system, UiSoundSystem},
    text::TextEditingMouseSystem,
    text_editing::TextEditingInputSystem,
    BlinkSystem, CachedSelectionOrderResource, UiButtonAction, UiEvent, UiLabel, UiPlaySoundAction,
    WidgetId, Widgets,
};

/// UI bundle
///
/// Will register all necessary components and systems needed for UI, along with any resources.
/// The generic type T represent the T generic parameter of the InputHandler<T>.
///
/// Will fail with error 'No resource with the given id' if either the InputBundle or TransformBundle are not added.
#[derive(new, Debug, Default)]
pub struct UiBundle</* C = NoCustomUi, */ W = u32, G = ()> {
    #[new(default)]
    _marker: PhantomData<(/* C, */ W, G)>,
}

impl</* C, */ W, G> SystemBundle for UiBundle</* C, */ W, G>
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
        log::debug!("Adding UI Resources");
        resources.insert(EventChannel::<UiButtonAction>::new());
        resources.insert(EventChannel::<UiEvent>::new());
        resources.insert(Widgets::<UiLabel, W>::new());
        resources.insert(CachedSelectionOrderResource::default());

        resources.insert(ProcessingQueue::<GlyphTextureData>::default());
        builder.add_system(GlyphTextureProcessorSystem::<DefaultBackend>::default());

        log::debug!("Creating UI EventChannel Readers");
        let ui_btn_reader = resources
            .get_mut::<EventChannel<UiButtonAction>>()
            .unwrap()
            .register_reader();

        let ui_btn_action_retrigger_reader = resources
            .get_mut::<EventChannel<UiEvent>>()
            .unwrap()
            .register_reader();

        let text_editing_mouse_reader = resources
            .get_mut::<EventChannel<Event<'static, ()>>>()
            .unwrap()
            .register_reader();

        let selection_mouse_reader = resources
            .get_mut::<EventChannel<UiEvent>>()
            .unwrap()
            .register_reader();

        let selection_keyboard_reader = resources
            .get_mut::<EventChannel<Event<'static, ()>>>()
            .unwrap()
            .register_reader();
        let text_editing_input_reader = resources
            .get_mut::<EventChannel<Event<'static, ()>>>()
            .unwrap()
            .register_reader();
        let drag_widget_reader = resources
            .get_mut::<EventChannel<UiEvent>>()
            .unwrap()
            .register_reader();

        log::debug!("Adding UI Systems to Dispatcher");
        builder
            .add_system(UiTransformSystem::new())
            .add_system(UiMouseSystem::new())
            .add_system(UiButtonSystem::new(ui_btn_reader))
            .add_system(ui_button_action_retrigger_event_system(
                ui_btn_action_retrigger_reader,
            ))
            .add_system(CacheSelectionSystem::<G>::new())
            .add_system(TextEditingMouseSystem::new(text_editing_mouse_reader))
            .add_system(SelectionMouseSystem::<G>::new(selection_mouse_reader))
            .add_system(SelectionKeyboardSystem::<G>::new(selection_keyboard_reader))
            .add_system(TextEditingInputSystem::new(text_editing_input_reader))
            .add_system(ResizeSystem::new())
            .add_system(DragWidgetSystem::new(drag_widget_reader))
            .add_system(BlinkSystem);

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
            .add_system(UiSoundSystem::new(
                resources
                    .get_mut::<EventChannel<UiPlaySoundAction>>()
                    .unwrap()
                    .register_reader(),
            ))
            .add_system(ui_sound_event_retrigger_system(
                resources
                    .get_mut::<EventChannel<UiEvent>>()
                    .unwrap()
                    .register_reader(),
            ));
        Ok(())
    }

    fn unload(&mut self, _world: &mut World, _resources: &mut Resources) -> Result<(), Error> {
        // FIXME: should get all resources and remove them
        Ok(())
    }
}
