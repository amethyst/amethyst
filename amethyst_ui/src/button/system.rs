use amethyst_assets::AssetStorage;
use amethyst_audio::output::Output;
use amethyst_audio::Source;
use amethyst_core::shrev::{EventChannel, ReaderId};
use amethyst_core::specs::{
    Entity, Read, ReadExpect, ReadStorage, Resources, System, SystemData, Write, WriteStorage,
};
use amethyst_core::ParentHierarchy;
use {UiButton, UiEvent, UiEventType::*, UiImage, UiText};

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
            mut image_storage,
            mut text_storage,
            events,
            audio_storage,
            audio_output,
            hierarchy,
        ): Self::SystemData,
    ) {
        let event_reader = self.event_reader.as_mut().unwrap();

        for event in events.read(event_reader) {
            let button = button_storage.get(event.target);
            let image = image_storage.get_mut(event.target);
            if button.is_none() || image.is_none() {
                continue;
            }
            let button = button.unwrap();
            let image = image.unwrap();
            match event.event_type {
                ClickStart => {
                    if let Some(texture) = button.press_image.as_ref() {
                        image.texture = texture.clone();
                    }
                    if button.press_text_color.is_none() {
                        continue;
                    }
                    for &child in hierarchy.children(event.target).unwrap_or(&[]) {
                        if let Some(text) = text_storage.get_mut(child) {
                            text.color = button.press_text_color.unwrap();
                        }
                    }
                }
                Click => {
                    if button.press_sound.is_none() || audio_output.is_none() {
                        continue;
                    }
                    let output = audio_output.as_ref().unwrap();
                    if let Some(sound) = audio_storage.get(button.press_sound.as_ref().unwrap()) {
                        output.play_once(sound, 1.0);
                    }
                }
                ClickStop => {
                    // This if statement is here to handle a situation
                    // where the user clicked on the button and with the mouse
                    // still held down, dragged their cursor off the button,
                    // and then released it.
                    //
                    // In this scenario we want to revert to the normal image
                    // however if the mouse is still over the button we want to
                    // use the hover image instead.
                    if Some(event.target) == self.hovered {
                        if let Some(hover_texture) = button.hover_image.as_ref() {
                            image.texture = hover_texture.clone();
                        } else {
                            image.texture = button.normal_image.clone();
                        }
                    } else {
                        image.texture = button.normal_image.clone();
                    }
                    if button.press_text_color.is_none() {
                        continue;
                    }
                    for &child in hierarchy.children(event.target).unwrap_or(&[]) {
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
                HoverStart => {
                    self.hovered = Some(event.target);
                    if let (Some(hover_sound), Some(audio_output)) = (
                        button
                            .hover_sound
                            .as_ref()
                            .and_then(|s| audio_storage.get(s)),
                        audio_output.as_ref(),
                    ) {
                        audio_output.play_once(hover_sound, 1.0);
                        println!("Played!");
                    }
                    if let Some(texture) = button.hover_image.as_ref() {
                        image.texture = texture.clone();
                    }
                    if button.hover_text_color.is_none() {
                        continue;
                    }
                    for &child in hierarchy.children(event.target).unwrap_or(&[]) {
                        if let Some(text) = text_storage.get_mut(child) {
                            text.color = button.hover_text_color.unwrap();
                        }
                    }
                }
                HoverStop => {
                    self.hovered = None;
                    image.texture = button.normal_image.clone();
                    for &child in hierarchy.children(event.target).unwrap_or(&[]) {
                        if let Some(text) = text_storage.get_mut(child) {
                            text.color = button.normal_text_color;
                        }
                    }
                }
            }
        }
    }
}
