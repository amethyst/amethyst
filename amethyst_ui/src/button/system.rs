use amethyst_assets::AssetStorage;
use amethyst_audio::{output::Output, Source};
use amethyst_core::{
    shrev::{EventChannel, ReaderId},
    specs::{
        Entity, Read, ReadExpect, ReadStorage, Resources, System, SystemData, Write, WriteStorage,
    },
    ParentHierarchy,
};

use crate::{OnUiActionImage, OnUiActionSound, UiButton, UiEvent, UiEventType::*, UiImage, UiText};

/// This system manages button mouse events.  It changes images and text colors, as well as playing audio
/// when necessary.
///
/// It's automatically registered with the `UiBundle`.
#[derive(Default)]
pub struct UiButtonSystem {
    event_reader: Option<ReaderId<UiEvent>>,
    hovered: Option<Entity>,
}

impl UiButtonSystem {
    /// Creates a new instance of this structure
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'s> System<'s> for UiButtonSystem {
    type SystemData = (
        ReadStorage<'s, UiButton>,
        ReadStorage<'s, OnUiActionImage>,
        ReadStorage<'s, OnUiActionSound>,
        WriteStorage<'s, UiImage>,
        WriteStorage<'s, UiText>,
        Write<'s, EventChannel<UiEvent>>,
        Read<'s, AssetStorage<Source>>,
        Option<Read<'s, Output>>,
        ReadExpect<'s, ParentHierarchy>,
    );

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.event_reader = Some(res.fetch_mut::<EventChannel<UiEvent>>().register_reader());
    }

    fn run(
        &mut self,
        (
            button_storage,
            action_image,
            action_sound,
            mut image_storage,
            mut text_storage,
            events,
            audio_storage,
            audio_output,
            hierarchy,
        ): Self::SystemData,
    ) {
        let event_reader = self
            .event_reader
            .as_mut()
            .expect("`UiButtonSystem::setup` was not called before `UiButtonSystem::run`");

        for event in events.read(event_reader) {
            let button = button_storage.get(event.target);
            let action_image = action_image.get(event.target);
            let action_sound = action_sound.get(event.target);
            match event.event_type {
                ClickStart => {
                    if let Some(action_image) = action_image {
                        if let Some(press_image) = action_image.press_image.as_ref() {
                            let _ = image_storage.insert(
                                event.target,
                                UiImage {
                                    texture: press_image.clone(),
                                },
                            );
                        } else {
                            image_storage.remove(event.target);
                        }
                    }

                    for &child in hierarchy.children(event.target) {
                        if let Some(text) = text_storage.get_mut(child) {
                            if let Some(new_color) = button.and_then(|b| b.press_text_color) {
                                text.color = new_color;
                            }
                        }
                    }

                    if let Some(output) = audio_output.as_ref() {
                        if let Some(sound) = action_sound
                            .and_then(|s| s.press_sound.as_ref())
                            .and_then(|s| audio_storage.get(s))
                        {
                            output.play_once(sound, 1.0);
                        }
                    }
                }
                Click => {
                    if let Some(output) = audio_output.as_ref() {
                        if let Some(sound) = action_sound
                            .and_then(|s| s.release_sound.as_ref())
                            .and_then(|s| audio_storage.get(s))
                        {
                            output.play_once(sound, 1.0);
                        }
                    }
                }
                ClickStop => {
                    if let Some(action_image) = action_image {
                        if Some(event.target) == self.hovered {
                            if let Some(hover_texture) = action_image.hover_image.as_ref() {
                                let _ = image_storage.insert(
                                    event.target,
                                    UiImage {
                                        texture: hover_texture.clone(),
                                    },
                                );
                            } else {
                                image_storage.remove(event.target);
                            }
                        } else {
                            if let Some(normal_image) = action_image.normal_image.as_ref() {
                                let _ = image_storage.insert(
                                    event.target,
                                    UiImage {
                                        texture: normal_image.clone(),
                                    },
                                );
                            } else {
                                image_storage.remove(event.target);
                            }
                        }
                    }

                    if let Some(button) = button {
                        for &child in hierarchy.children(event.target) {
                            if let Some(text) = text_storage.get_mut(child) {
                                if Some(event.target) == self.hovered {
                                    if let Some(hover_color) = button.hover_text_color {
                                        text.color = hover_color;
                                    } else {
                                        text.color = button.normal_text_color;
                                    }
                                } else {
                                    text.color = button.normal_text_color;
                                }
                            }
                        }
                    }
                }
                HoverStart => {
                    self.hovered = Some(event.target);
                    if let (Some(hover_sound), Some(audio_output)) = (
                        action_sound
                            .and_then(|s| s.hover_sound.as_ref())
                            .and_then(|s| audio_storage.get(s)),
                        audio_output.as_ref(),
                    ) {
                        audio_output.play_once(hover_sound, 1.0);
                    }
                    if let Some(image) = image_storage.get_mut(event.target) {
                        if let Some(texture) = action_image.and_then(|i| i.hover_image.as_ref()) {
                            image.texture = texture.clone();
                        }
                    }
                    if let Some(button) = button {
                        for &child in hierarchy.children(event.target) {
                            if let Some(text) = text_storage.get_mut(child) {
                                if let Some(new_color) = button.hover_text_color {
                                    text.color = new_color;
                                }
                            }
                        }
                    }
                }
                HoverStop => {
                    self.hovered = None;
                    if let Some(action_image) = action_image {
                        if let Some(normal_image) = action_image.normal_image.as_ref() {
                            let _ = image_storage.insert(
                                event.target,
                                UiImage {
                                    texture: normal_image.clone(),
                                },
                            );
                        } else {
                            image_storage.remove(event.target);
                        }
                    }
                    if let Some(button) = button {
                        for &child in hierarchy.children(event.target) {
                            if let Some(text) = text_storage.get_mut(child) {
                                text.color = button.normal_text_color;
                            }
                        }
                    }
                }
            }
        }
    }
}
