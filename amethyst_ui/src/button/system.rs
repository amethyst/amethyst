use amethyst_core::{
    ecs::{Entity, ReadExpect, Resources, System, SystemData, Write, WriteStorage},
    shrev::{EventChannel, ReaderId},
    ParentHierarchy,
};
use std::collections::HashMap;

use crate::{UiButtonAction, UiButtonActionType::*, UiImageComponent, UiTextComponent};

struct ActionChangeStack<T: Clone + PartialEq> {
    initial_value: T,
    stack: Vec<T>,
}

impl<T> ActionChangeStack<T>
where
    T: Clone + PartialEq,
{
    pub fn new(initial_value: T) -> Self {
        ActionChangeStack {
            initial_value,
            stack: Vec::new(),
        }
    }

    pub fn add(&mut self, change: T) {
        self.stack.push(change);
    }

    pub fn remove(&mut self, change: &T) -> Option<T> {
        if let Some(idx) = self.stack.iter().position(|it| it == change) {
            Some(self.stack.remove(idx))
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn current(&self) -> T {
        if self.stack.is_empty() {
            self.initial_value.clone()
        } else {
            self.stack
                .iter()
                .last()
                .map(T::clone)
                .expect("Unreachable: Just checked that stack is not empty")
        }
    }
}

/// This system manages button mouse events.  It changes images and text colors, as well as playing audio
/// when necessary.
///
/// It's automatically registered with the `UiBundle`.
#[derive(Default)]
pub struct UiButtonSystem {
    event_reader: Option<ReaderId<UiButtonAction>>,
    set_images: HashMap<Entity, ActionChangeStack<UiImageComponent>>,
    set_text_colors: HashMap<Entity, ActionChangeStack<[f32; 4]>>,
}

impl UiButtonSystem {
    /// Creates a new instance of this structure
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'s> System<'s> for UiButtonSystem {
    type SystemData = (
        WriteStorage<'s, UiImageComponent>,
        WriteStorage<'s, UiTextComponent>,
        ReadExpect<'s, ParentHierarchy>,
        Write<'s, EventChannel<UiButtonAction>>,
    );

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.event_reader = Some(
            res.fetch_mut::<EventChannel<UiButtonAction>>()
                .register_reader(),
        );
    }

    fn run(
        &mut self,
        (mut image_storage, mut text_storage, hierarchy, button_events): Self::SystemData,
    ) {
        let event_reader = self
            .event_reader
            .as_mut()
            .expect("`UiButtonSystem::setup` was not called before `UiButtonSystem::run`");

        for event in button_events.read(event_reader) {
            match event.event_type {
                SetTextColor(ref color) => {
                    for &child in hierarchy.children(event.target) {
                        if let Some(text) = text_storage.get_mut(child) {
                            // found the text. push its original color if
                            // it's not there yet
                            self.set_text_colors
                                .entry(event.target)
                                .or_insert_with(|| ActionChangeStack::new(text.color))
                                .add(*color);

                            text.color = *color;
                        }
                    }
                }
                UnsetTextColor(ref color) => {
                    for &child in hierarchy.children(event.target) {
                        if let Some(text) = text_storage.get_mut(child) {
                            // first, remove the color we were told to unset
                            if !self.set_text_colors.contains_key(&event.target) {
                                // nothing to do!
                                continue;
                            }

                            self.set_text_colors
                                .get_mut(&event.target)
                                .and_then(|it| it.remove(color));

                            text.color = self.set_text_colors[&event.target].current();

                            if self.set_text_colors[&event.target].is_empty() {
                                self.set_text_colors.remove(&event.target);
                            }
                        }
                    }
                }
                SetImage(ref set_image) => {
                    if let Some(image) = image_storage.get_mut(event.target) {
                        self.set_images
                            .entry(event.target)
                            .or_insert_with(|| ActionChangeStack::new(image.clone()))
                            .add(set_image.clone());

                        *image = set_image.clone();
                    }
                }
                UnsetTexture(ref unset_image) => {
                    if let Some(image) = image_storage.get_mut(event.target) {
                        if !self.set_images.contains_key(&event.target) {
                            continue;
                        }

                        self.set_images
                            .get_mut(&event.target)
                            .and_then(|it| it.remove(unset_image));

                        *image = self.set_images[&event.target].current();

                        if self.set_images[&event.target].is_empty() {
                            self.set_images.remove(&event.target);
                        }
                    }
                }
            };
        }
    }
}
