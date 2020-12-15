use std::{
    collections::{HashMap, HashSet}
};

use amethyst_core::{
    ecs::*,
    math::Vector2,
    shrev::EventChannel,
    Hidden, HiddenPropagate,
};
use amethyst_input::InputHandler;
use amethyst_window::ScreenDimensions;
use serde::{Deserialize, Serialize};

use crate::{
    get_parent_pixel_size, targeted_below, Interactable, ScaleMode, UiEvent, UiEventType,
    UiTransform,
};
use amethyst_core::shrev::ReaderId;
use amethyst_core::ecs::{SystemBuilder, IntoQuery};
use amethyst_core::transform::Parent;

/// Component that denotes whether a given ui widget is draggable.
/// Requires UiTransform to work, and its expected way of usage is
/// through UiTransformData prefab.
#[derive(Debug, Serialize, Deserialize)]
pub struct Draggable;

#[derive(Debug)]
pub struct DragWidgetSystemResource {
    event_reader: ReaderId<UiEvent>,

    /// hashmap whose keys are every entities being dragged,
    /// and whose element is a tuple whose first element is
    /// the original mouse position when drag first started,
    /// and second element the mouse position one frame ago
    record: HashMap<Entity, (Vector2<f32>, Vector2<f32>)>,
}

impl DragWidgetSystemResource {
    pub fn new(event_reader: ReaderId<UiEvent>) -> Self {
        Self {
            event_reader,
            record: HashMap::new(),
        }
    }
}

pub fn build_drag_widget_system(resources: &mut Resources) -> impl Runnable {
    let reader_id = resources.get_mut::<EventChannel<UiEvent>>().unwrap().register_reader();
    resources.insert(DragWidgetSystemResource::new(reader_id));
    SystemBuilder::new("DragWidgetSystem")
            .write_resource::<DragWidgetSystemResource>()
            .write_resource::<EventChannel<UiEvent>>()
            .read_resource::<InputHandler>()
            .read_resource::<ScreenDimensions>()
            .with_query(<(Entity, &Draggable)>::query())
            .with_query(<&Hidden>::query())
            .with_query(<&HiddenPropagate>::query())
            .with_query(<Option<&Parent>>::query())
            .with_query(<(Entity, Option<&UiTransform>)>::query())
            .with_query(<&mut UiTransform>::query())
            .with_query(<(Entity, &UiTransform, Option<&Interactable>)>::query()
                .filter(!component::<Hidden>() & !component::<HiddenPropagate>()))
            .build(move |_commands, world, (resource, ui_events, input, screen_dimensions),
                         (draggables, hiddens, hidden_props, maybe_parent, maybe_ui_transform, ui_transforms, not_hidden_ui_transforms)| {
                let mouse_pos = input.mouse_position().unwrap_or((0., 0.));
                let mouse_pos = Vector2::new(mouse_pos.0, screen_dimensions.height() - mouse_pos.1);
                let mut click_stopped: HashSet<Entity> = HashSet::new();
                let event_reader = &mut resource.event_reader;
                ui_events.read(event_reader).for_each(|event| match event.event_type {
                    UiEventType::ClickStart => {
                        if draggables.iter(world).any(|(e, _)| *e == event.target) {
                            resource.record.insert(event.target, (mouse_pos, mouse_pos));
                        }
                    }
                    UiEventType::ClickStop => {
                        if resource.record.contains_key(&event.target) {
                            click_stopped.insert(event.target);
                        }
                    }
                    _ => (),
                });

                for (entity, _) in resource.record.iter() {
                    if hiddens.get(world, *entity).is_ok() || hidden_props.get(world, *entity).is_ok() {
                        click_stopped.insert(*entity);
                    }
                }

                for (entity, (first, mut prev)) in resource.record.iter_mut() {
                    ui_events.single_write(UiEvent::new(
                        UiEventType::Dragging {
                            offset_from_mouse: mouse_pos - first.clone(),
                            new_position: mouse_pos,
                        },
                        *entity,
                    ));

                    let change = mouse_pos - prev.clone();


                    let (parent_width, parent_height) = {
                        let maybe_parent_current = maybe_parent.get(world, *entity).unwrap();
                        let maybe_transform_iter = maybe_ui_transform.iter(world);
                        get_parent_pixel_size(maybe_parent_current, maybe_transform_iter, &screen_dimensions)
                    };


                    let ui_transform = ui_transforms.get_mut(world, *entity).unwrap();
                    let (scale_x, scale_y) = match ui_transform.scale_mode {
                        ScaleMode::Pixel => (1.0, 1.0),
                        ScaleMode::Percent => (parent_width, parent_height),
                    };

                    ui_transform.local_x += change[0] / scale_x;
                    ui_transform.local_y += change[1] / scale_y;

                    *prev = *mouse_pos;
                }

                for entity in click_stopped.iter() {
                    ui_events.single_write(UiEvent::new(
                        UiEventType::Dropped {
                            dropped_on: targeted_below(
                                (mouse_pos[0], mouse_pos[1]),
                                ui_transforms.get_mut(world, *entity).unwrap().global_z,
                                not_hidden_ui_transforms.iter(world)
                            ),
                        },
                        *entity,
                    ));

                    resource.record.remove(entity);
                }
            })
}
