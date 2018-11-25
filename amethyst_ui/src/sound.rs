use amethyst_assets::AssetStorage;
use amethyst_audio::{output::Output, Source, SourceHandle};
use amethyst_core::{
    shrev::{EventChannel, ReaderId},
    specs::{
        prelude::{Component, DenseVecStorage},
        Read, Resources, System, SystemData, Write,
    },
};

use crate::{
    event::{UiEvent, UiEventType::*},
    event_retrigger::{EventRetrigger, EventRetriggerSystem},
    EventReceiver,
};

/// Provides an `EventRetriggerSystem` that will handle incoming `UiEvent`s
/// and trigger `UiPlaySoundAction`s for entities with attached
/// `UiSoundRetrigger` components.
pub type UiSoundRetriggerSystem = EventRetriggerSystem<UiSoundRetrigger>;

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

impl Component for UiSoundRetrigger {
    type Storage = DenseVecStorage<Self>;
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

/// Handles any dispatches `UiPlaySoundAction`s and plays the received
/// sounds through the set `Output`.
#[derive(Default)]
pub struct UiSoundSystem {
    event_reader: Option<ReaderId<UiPlaySoundAction>>,
}

impl UiSoundSystem {
    /// Constructs a default `UiSoundSystem`. Since the `event_reader`
    /// will automatically be fetched when the system is set up, this should
    /// always be used to construct the `UiSoundSystem`.
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'s> System<'s> for UiSoundSystem {
    type SystemData = (
        Write<'s, EventChannel<UiPlaySoundAction>>,
        Read<'s, AssetStorage<Source>>,
        Option<Read<'s, Output>>,
    );

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.event_reader = Some(
            res.fetch_mut::<EventChannel<UiPlaySoundAction>>()
                .register_reader(),
        );
    }

    fn run(&mut self, (sound_events, audio_storage, audio_output): Self::SystemData) {
        let event_reader = self
            .event_reader
            .as_mut()
            .expect("`UiSoundSystem::setup` was not called before `UiSoundSystem::run`");

        for event in sound_events.read(event_reader) {
            if let Some(output) = audio_output.as_ref() {
                if let Some(sound) = audio_storage.get(&event.0) {
                    output.play_once(sound, 1.0);
                }
            }
        }
    }
}
