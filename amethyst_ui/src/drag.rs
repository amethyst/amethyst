use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;

use amethyst_core::{
    ecs::{
        Component, DenseVecStorage, Entities, Entity, Join, Read, ReadExpect, ReadStorage,
        ReaderId, System, SystemData, Write, WriteStorage,
    },
    math::Vector2,
    shrev::EventChannel,
};
use amethyst_derive::SystemDesc;
use amethyst_input::{BindingTypes, InputHandler};
use amethyst_window::ScreenDimensions;

use crate::{targeted_below, Interactable, UiEvent, UiEventType, UiTransform};

/// Component that denotes whether a given ui widget is draggable.
/// Requires UiTransform to work, and its expected way of usage is
/// through UiTransformData prefab.
#[derive(Debug, Serialize, Deserialize)]
pub struct Draggable;

impl Component for Draggable {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, SystemDesc)]
#[system_desc(name(DragWidgetSystemDesc))]
pub struct DragWidgetSystem<T: BindingTypes> {
    #[system_desc(event_channel_reader)]
    ui_reader_id: ReaderId<UiEvent>,

    #[system_desc(skip)]
    record: HashMap<Entity, (Vector2<f32>, Vector2<f32>)>,

    phantom: PhantomData<T>,
}

impl<T> DragWidgetSystem<T>
where
    T: BindingTypes,
{
    pub fn new(ui_reader_id: ReaderId<UiEvent>) -> Self {
        Self {
            ui_reader_id,
            record: HashMap::new(),
            phantom: PhantomData,
        }
    }
}

impl<'s, T> System<'s> for DragWidgetSystem<T>
where
    T: BindingTypes,
{
    type SystemData = (
        Entities<'s>,
        Read<'s, InputHandler<T>>,
        ReadExpect<'s, ScreenDimensions>,
        ReadStorage<'s, Draggable>,
        ReadStorage<'s, Interactable>,
        Write<'s, EventChannel<UiEvent>>,
        WriteStorage<'s, UiTransform>,
    );

    fn run(
        &mut self,
        (
            entities,
            input_handler,
            screen_dimensions,
            draggables,
            interactables,
            mut ui_events,
            mut ui_transforms,
        ): Self::SystemData,
    ) {
        if let Some((mouse_x, mouse_y)) = input_handler.mouse_position() {
            let mouse_pos = Vector2::new(mouse_x, screen_dimensions.height() - mouse_y);

            let mut click_stopped: Vec<Entity> = Vec::new();

            for event in ui_events.read(&mut self.ui_reader_id) {
                match event.event_type {
                    UiEventType::ClickStart => {
                        if draggables.get(event.target).is_some() {
                            self.record.insert(event.target, (mouse_pos, mouse_pos));
                        }
                    }
                    UiEventType::ClickStop => {
                        if draggables.get(event.target).is_some() {
                            click_stopped.push(event.target);
                        }
                    }
                    _ => (),
                }
            }

            for (entity, (first, prev)) in self.record.iter_mut() {
                ui_events.single_write(UiEvent::new(
                    UiEventType::Dragging {
                        offset_from_mouse: mouse_pos - *first,
                        new_position: mouse_pos,
                    },
                    *entity,
                ));

                let ui_transform = ui_transforms.get_mut(*entity).unwrap();
                let change = mouse_pos - *prev;

                ui_transform.local_x += change[0];
                ui_transform.local_y += change[1];

                *prev = mouse_pos;
            }

            for entity in click_stopped.iter() {
                ui_events.single_write(UiEvent::new(
                    UiEventType::Dropped {
                        dropped_on: targeted_below(
                            (mouse_pos[0], mouse_pos[1]),
                            ui_transforms.get(*entity).unwrap().global_z,
                            (&*entities, &ui_transforms, interactables.maybe()).join(),
                        ),
                    },
                    *entity,
                ));

                self.record.remove(entity);
            }
        }
    }
}
