use amethyst_assets::AssetStorage;
use amethyst_audio::{output::Output, Source, SourceHandle};
use amethyst_core::{
    ecs::*,
    shrev::{EventChannel, ReaderId},
};
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{
    event::{UiEvent, UiEventType::*},
    event_retrigger::{build_event_retrigger_system, EventRetrigger},
    EventReceiver,
};

/// Provides an `EventRetriggerSystem` that will handle incoming `UiEvent`s
/// and trigger `UiPlaySoundAction`s for entities with attached
/// `UiSoundRetrigger` components.
pub fn build_ui_sound_retrigger_system(resources: &mut Resources) -> impl Runnable {
    build_event_retrigger_system::<UiSoundRetrigger>(resources)
}

/// Action that will trigger a sound to play in `UiSoundSystem`.
#[derive(Debug, Clone)]
pub struct UiPlaySoundAction(pub SourceHandle);

/// Attach this to an entity to play the respective sound when a `UiEvent`
/// targets the entity.
#[derive(Debug, Clone)]
pub struct UiSoundRetrigger {
    /// The sound that is played when the user begins a click on the entity
    pub on_click_start: Option<UiPlaySoundAction>,
    /// The sound that is played when the user ends a click on the entity
    pub on_click_stop: Option<UiPlaySoundAction>,
    /// The sound that is played when the user starts hovering over the entity
    pub on_hover_start: Option<UiPlaySoundAction>,
    /// The sound that is played when the user stops hovering over the entity
    pub on_hover_stop: Option<UiPlaySoundAction>,
}

impl EventRetrigger for UiSoundRetrigger {
    type In = UiEvent;
    type Out = UiPlaySoundAction;

    fn apply<R>(&self, event: &Self::In, out: &mut R)
    where
        R: EventReceiver<Self::Out>,
    {
        let event_to_trigger = match &event.event_type {
            ClickStart => &self.on_click_start,
            ClickStop => &self.on_click_stop,
            HoverStart => &self.on_hover_start,
            HoverStop => &self.on_hover_stop,
            _ => return,
        };

        if let Some(ev) = event_to_trigger {
            out.receive_one(&ev);
        }
    }
}

#[derive(Debug)]
pub struct UiSoundSystemResource {
    event_reader: ReaderId<UiPlaySoundAction>,
}

impl UiSoundSystemResource {
    /// Constructs a default `UiSoundSystem`. Since the `event_reader`
    /// will automatically be fetched when the system is set up, this should
    /// always be used to construct the `UiSoundSystem`.
    pub fn new(event_reader: ReaderId<UiPlaySoundAction>) -> Self {
        Self { event_reader }
    }
}

/// Handles any dispatches `UiPlaySoundAction`s and plays the received
/// sounds through the set `Output`.
pub fn build_ui_sound_system(resources: &mut Resources) -> impl Runnable {
    let reader_id = resources
        .get_mut::<EventChannel<UiPlaySoundAction>>()
        .unwrap()
        .register_reader();
    resources.insert(UiSoundSystemResource::new(reader_id));

    SystemBuilder::new("UiSoundSystem")
        .write_resource::<UiSoundSystemResource>()
        .write_resource::<EventChannel<UiPlaySoundAction>>()
        .read_resource::<AssetStorage<Source>>()
        .read_resource::<Output>()
        .build(
            move |_commands, _world, (resource, sound_events, audio_storage, audio_output), _| {
                #[cfg(feature = "profiler")]
                profile_scope!("ui_sound_system");
                let event_reader = &mut resource.event_reader;
                for event in sound_events.read(event_reader) {
                    if let Some(sound) = audio_storage.get(&event.0) {
                        audio_output.play_once(sound, 1.0);
                    }
                }
            },
        )
}
