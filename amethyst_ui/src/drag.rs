use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use amethyst_core::{
    ecs::{
        Component, DenseVecStorage, Entities, Entity, Join, Read, ReaderId, System, SystemData,
        Write, WriteStorage,
    },
    math::Vector2,
    shrev::EventChannel,
};
use amethyst_derive::SystemDesc;
use amethyst_input::{BindingTypes, InputEvent};

use crate::{UiEvent, UiEventType, UiTransform};

/// Component that denotes whether a given ui widget is draggable.
/// Requires UiTransform to work, and its expected way of usage is
/// through UiTransformData prefab.
#[derive(Debug, Serialize, Deserialize)]
pub struct Draggable {
    /// When true, the widget is being dragged. When false, it's not.
    pub being_dragged: bool,
}

impl Component for Draggable {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, SystemDesc)]
#[system_desc(name(DragWidgetSystemDesc))]
pub struct DragWidgetSystem<T: BindingTypes> {
    #[system_desc(event_channel_reader)]
    input_reader_id: ReaderId<InputEvent<T>>,

    #[system_desc(event_channel_reader)]
    ui_reader_id: ReaderId<UiEvent>,

    phantom: PhantomData<T>,
}

impl<T> DragWidgetSystem<T>
where
    T: BindingTypes,
{
    pub fn new(input_reader_id: ReaderId<InputEvent<T>>, ui_reader_id: ReaderId<UiEvent>) -> Self {
        Self {
            input_reader_id,
            ui_reader_id,
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
        Read<'s, EventChannel<InputEvent<T>>>,
        Write<'s, EventChannel<UiEvent>>,
        WriteStorage<'s, Draggable>,
        WriteStorage<'s, UiTransform>,
    );

    fn run(
        &mut self,
        (entities, input_events, mut ui_events, mut draggables, mut ui_transforms): Self::SystemData,
    ) {
        let mut click_stopped: Vec<Entity> = Vec::new();

        for event in ui_events.read(&mut self.ui_reader_id) {
            match event.event_type {
                UiEventType::ClickStart => {
                    if let Some(draggable) = draggables.get_mut(event.target) {
                        draggable.being_dragged = true;
                    }
                }
                UiEventType::ClickStop => {
                    if let Some(draggable) = draggables.get(event.target) {
                        if draggable.being_dragged {
                            click_stopped.push(event.target);
                        }
                    }
                }
                _ => (),
            }
        }

        for event in input_events.read(&mut self.input_reader_id) {
            if let InputEvent::CursorMoved { delta_x, delta_y } = event {
                for (entity, draggable, mut ui_transforms) in
                    (&entities, &draggables, &mut ui_transforms.restrict_mut()).join()
                {
                    if draggable.being_dragged {
                        let ui_transform = ui_transforms.get_mut_unchecked();

                        ui_events.single_write(UiEvent::new(
                            UiEventType::Dragging {
                                element_offset: Vector2::new(*delta_x, -*delta_y),
                            },
                            entity,
                        ));
                        ui_transform.local_x += delta_x;
                        ui_transform.local_y -= delta_y;
                    }
                }
            }
        }

        for entity in click_stopped.iter() {
            draggables.get_mut(*entity).unwrap().being_dragged = false;

            ui_events.single_write(UiEvent::new(
                UiEventType::Dropped {
                    dropped_on: *entity,
                },
                *entity,
            ));
        }
    }
}
