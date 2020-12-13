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
use amethyst_core::ecs::systems::ParallelRunnable;
use amethyst_core::ecs::{SystemBuilder, IntoQuery};
use amethyst_core::transform::Parent;

/// Component that denotes whether a given ui widget is draggable.
/// Requires UiTransform to work, and its expected way of usage is
/// through UiTransformData prefab.
#[derive(Debug, Serialize, Deserialize)]
pub struct Draggable;

#[derive(Debug)]
pub struct DragWidgetSystem {
    event_reader: ReaderId<UiEvent>,

    /// hashmap whose keys are every entities being dragged,
    /// and whose element is a tuple whose first element is
    /// the original mouse position when drag first started,
    /// and second element the mouse position one frame ago
    record: HashMap<Entity, (Vector2<f32>, Vector2<f32>)>,
}

impl DragWidgetSystem {
    pub fn new(event_reader: ReaderId<UiEvent>) -> Self {
        Self {
            event_reader,
            record: HashMap::new(),
        }
    }

    pub fn build(&mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("DragWidgetSystem")
                .write_resource::<EventChannel<UiEvent>>()
                .read_resource::<InputHandler>()
                .read_resource::<ScreenDimensions>()
                .with_query(<&Draggable>::query())
                .with_query(<&Hidden>::query())
                .with_query(<&HiddenPropagate>::query())
                .with_query(<Option<&Parent>>::query())
                .with_query(<Option<&UiTransform>>::query())
                .with_query(<&mut UiTransform>::query())
                .with_query(<(Entity, &UiTransform, Option<&Interactable>)>::query()
                    .filter(!component::<Hidden>() & !component::<HiddenPropagate>()))
                .build(move |_commands, world, (ui_events, input, screen_dimensions),
                             (draggables, hiddens, hidden_props, maybe_parent, maybe_ui_transform, ui_transforms, not_hidden_ui_transforms)| {
                    let mouse_pos = input.mouse_position().unwrap_or((0., 0.));
                    let mouse_pos = Vector2::new(mouse_pos.0, screen_dimensions.height() - mouse_pos.1);
                    let mut click_stopped: HashSet<Entity> = HashSet::new();
                    let event_reader = &mut self.event_reader;
                    ui_events.read(event_reader).for_each(|event| match event {
                        UiEventType::ClickStart => {
                            if draggables.iter(world).any(|e, _| e == event.get_target()) {
                                self.record.insert(event.get_target(), (mouse_pos, mouse_pos));
                            }
                        }
                        UiEventType::ClickStop => {
                            if self.record.contains_key(&event.get_target()) {
                                click_stopped.insert(event.get_target());
                            }
                        }
                        _ => (),
                    });

                    for (entity, _) in self.record.iter() {
                        if hiddens.iter(world).any(|e, _| e == entity) || hidden_props.any(|e, _| e == entity) {
                            click_stopped.insert(*entity);
                        }
                    }

                    for (entity, (first, prev)) in self.record.iter_mut() {
                        ui_events.single_write(UiEvent::new(
                            UiEventType::Dragging {
                                offset_from_mouse: mouse_pos - first,
                                new_position: mouse_pos,
                            },
                            *entity,
                        ));

                        let change = mouse_pos - prev;

                        let (parent_width, parent_height) =
                            get_parent_pixel_size(maybe_parent.filter(|e, _| e == entity).next(), maybe_ui_transform, &screen_dimensions);

                        let ui_transform = ui_transforms.iter_mut(world).filter(|e, t| e == entity).next().unwrap();
                        let (scale_x, scale_y) = match ui_transform.scale_mode {
                            ScaleMode::Pixel => (1.0, 1.0),
                            ScaleMode::Percent => (parent_width, parent_height),
                        };

                        ui_transform.local_x += change[0] / scale_x;
                        ui_transform.local_y += change[1] / scale_y;

                        *prev = mouse_pos;
                    }

                    for entity in click_stopped.iter() {
                        ui_events.single_write(UiEvent::new(
                            UiEventType::Dropped {
                                dropped_on: targeted_below(
                                    (mouse_pos[0], mouse_pos[1]),
                                    ui_transforms.iter_mut(world).filter(|e, t| e == entity).next().unwrap().global_z,
                                    not_hidden_ui_transforms.iter(world)
                                ),
                            },
                            *entity,
                        ));

                        self.record.remove(entity);
                    }
                })
        )
    }
}