use std::{collections::HashMap, fmt::Debug};

use amethyst_core::{
    ecs::*,
    shrev::{EventChannel, ReaderId},
};

use crate::{UiButtonAction, UiButtonActionType::*, UiImage, UiText};
use amethyst_core::ecs::systems::ParallelRunnable;
use amethyst_core::transform::Parent;

#[derive(Debug)]
struct ActionChangeStack<T: Debug + Clone + PartialEq> {
    initial_value: T,
    stack: Vec<T>,
}

impl<T> ActionChangeStack<T>
where
    T: Debug + Clone + PartialEq,
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
#[derive(Debug)]
pub struct UiButtonSystem {
    event_reader: ReaderId<UiButtonAction>,
    set_images: HashMap<Entity, ActionChangeStack<UiImage>>,
    set_text_colors: HashMap<Entity, ActionChangeStack<[f32; 4]>>,
}

impl UiButtonSystem {
    /// Creates a new instance of this structure
    pub fn new(event_reader: ReaderId<UiButtonAction>) -> Self {
        Self {
            event_reader,
            set_images: Default::default(),
            set_text_colors: Default::default(),
        }
    }

    pub fn build(&mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("UiButtonSystem")
                .write_resource::<EventChannel<UiButtonAction>>()
                .with_query(<(&Parent, Write<&UiText>)>::query())
                .with_query(<Write<&UiImage>>::query())
                .build( move | _commands, world, button_events, (children_with_text, images )| {
                    let event_reader = &mut self.event_reader;
                    for event in button_events.read(event_reader) {
                        match event.event_type {
                            SetTextColor(ref color) => {
                                children_with_text.for_each_mut(world, |entity, parent, mut text| {
                                    if entity == event.get_target() {
                                        self.set_text_colors
                                            .entry(event.target)
                                            .or_insert_with(|| ActionChangeStack::new(text.color))
                                            .add(*color);

                                        text.color = *color;
                                    }
                                });
                            }
                            UnsetTextColor(ref color) => {
                                children_with_text.for_each_mut(world, |entity, parent, mut text| {
                                    if entity == event.get_target() {
                                        if self.set_text_colors.contains_key(&event.target) {
                                            self.set_text_colors
                                                .get_mut(&event.target)
                                                .and_then(|it| it.remove(color));

                                            text.color = self.set_text_colors[&event.target].current();

                                            if self.set_text_colors[&event.target].is_empty() {
                                                self.set_text_colors.remove(&event.get_target());
                                            }
                                        }
                                    }
                                });
                            }
                            SetImage(ref set_image) => {
                                images.for_each_mut(world, |entity, image| {
                                    if event.get_target() == entity {
                                        self.set_images
                                            .entry(event.target)
                                            .or_insert_with(|| ActionChangeStack::new(image.clone()))
                                            .add(set_image.clone());

                                        *image = set_image.clone();
                                    }
                                });
                            }
                            UnsetTexture(ref unset_image) => {
                                images.for_each_mut(world, |entity, image| {
                                    if event.get_target() == entity {
                                        if self.set_images.contains_key(&event.target) {
                                            self.set_images
                                                .get_mut(&event.target)
                                                .and_then(|it| it.remove(unset_image));

                                            *image = self.set_images[&event.target].current();

                                            if self.set_images[&event.target].is_empty() {
                                                self.set_images.remove(&event.target);
                                            }
                                        }
                                    }
                                });
                            }
                        };
                    }
                })
        )
    }
}
