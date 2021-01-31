use std::collections::HashSet;

use amethyst_core::{ecs::*, math::Vector2, shrev::EventChannel, Hidden, HiddenPropagate};
use amethyst_input::InputHandler;
use amethyst_window::ScreenDimensions;
use serde::{Deserialize, Serialize};
use winit::event::MouseButton;

use crate::transform::UiTransform;

/// An event that pertains to a specific `Entity`, for example a `UiEvent` for clicking on a widget
/// entity.
pub trait TargetedEvent {
    /// The `Entity` targeted by the event.
    fn get_target(&self) -> Entity;
}

/// The type of ui event.
/// Click happens if you start and stop clicking on the same ui element.
#[derive(Debug, Clone, PartialEq)]
pub enum UiEventType {
    /// When an element is clicked normally.
    /// Includes touch events.
    Click,
    /// When the element starts being clicked (On left mouse down).
    /// Includes touch events.
    ClickStart,
    /// When the element stops being clicked (On left mouse up).
    /// Includes touch events.
    ClickStop,
    /// When the cursor gets over an element.
    HoverStart,
    /// When the cursor stops being over an element.
    HoverStop,
    /// When dragging a `Draggable` Ui element.
    Dragging {
        /// The position of the mouse relative to the center of the transform when the drag started.
        offset_from_mouse: Vector2<f32>,
        /// Position at which the mouse is currently. Absolute value; not relative to the parent of the dragged entity.
        new_position: Vector2<f32>,
    },
    /// When stopping to drag a `Draggable` Ui element.
    Dropped {
        /// The entity on which the dragged object was dropped.
        dropped_on: Option<Entity>,
    },
    /// When the value of a UiText element has been changed by user input.
    ValueChange,
    /// When the value of a UiText element has been committed by user action.
    ValueCommit,
    /// When an editable UiText element has gained focus.
    Focus,
    /// When an editable UiText element has lost focus.
    Blur,
}

/// A ui event instance.
#[derive(Debug, Clone)]
pub struct UiEvent {
    /// The type of ui event.
    pub event_type: UiEventType,
    /// The entity on which the event happened.
    pub target: Entity,
}

impl UiEvent {
    /// Creates a new UiEvent.
    pub fn new(event_type: UiEventType, target: Entity) -> Self {
        UiEvent { event_type, target }
    }
}

impl TargetedEvent for UiEvent {
    fn get_target(&self) -> Entity {
        self.target
    }
}

/// A component that tags an entity as reactive to ui events.
/// Will only work if the entity has a UiTransform component attached to it.
/// Without this, the ui element will not generate events.
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Interactable;

/// The system that generates events for `Interactable` enabled entities.
/// The generic types A and B represent the A and B generic parameter of the InputHandler<A,B>.

#[derive(Default, Debug)]
pub struct UiMouseSystem {
    was_down: bool,
    click_started_on: HashSet<Entity>,
    last_targets: HashSet<Entity>,
}

impl UiMouseSystem {
    /// Creates a new UiMouseSystem.
    pub fn new() -> Self {
        UiMouseSystem {
            was_down: false,
            click_started_on: HashSet::new(),
            last_targets: HashSet::new(),
        }
    }
}

impl System for UiMouseSystem {
    fn build(mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("UiMouseSystem")
                .write_resource::<EventChannel<UiEvent>>()
                .read_resource::<InputHandler>()
                .read_resource::<ScreenDimensions>()
                // Interactable entities with an UiTransform, without Hidden or HiddenPropagate
                .with_query(
                    <(Entity, &UiTransform, Option<&Interactable>)>::query()
                        .filter(!component::<Hidden>() & !component::<HiddenPropagate>()),
                )
                .build(
                    move |_commands,
                          world,
                          (events, input, screen_dimensions),
                          interactables_entities| {
                        let down = input.mouse_button_is_down(MouseButton::Left);
                        // FIXME: To replace on InputHandler generate OnMouseDown and OnMouseUp events See #2496
                        let click_started = down && !self.was_down;
                        let click_stopped = !down && self.was_down;
                        if let Some((pos_x, pos_y)) = input.mouse_position() {
                            let x = pos_x as f32;
                            let y = screen_dimensions.height() - pos_y as f32;

                            let targets = targeted((x, y), interactables_entities.iter(world));

                            for target in targets.difference(&self.last_targets) {
                                events.single_write(UiEvent::new(UiEventType::HoverStart, *target));
                            }

                            for last_target in self.last_targets.difference(&targets) {
                                events.single_write(UiEvent::new(
                                    UiEventType::HoverStop,
                                    *last_target,
                                ));
                            }

                            if click_started {
                                self.click_started_on = targets.clone();
                                for target in targets.iter() {
                                    events.single_write(UiEvent::new(
                                        UiEventType::ClickStart,
                                        *target,
                                    ));
                                }
                            } else if click_stopped {
                                for click_start_target in
                                    self.click_started_on.intersection(&targets)
                                {
                                    events.single_write(UiEvent::new(
                                        UiEventType::Click,
                                        *click_start_target,
                                    ));
                                }
                            }

                            self.last_targets = targets;
                        }

                        // Could be used for drag and drop
                        if click_stopped {
                            for click_start_target in self.click_started_on.drain() {
                                events.single_write(UiEvent::new(
                                    UiEventType::ClickStop,
                                    click_start_target,
                                ));
                            }
                        }
                        self.was_down = down;
                    },
                ),
        )
    }
}

/// Finds all interactable entities at the position `pos` which don't have any opaque entities on
/// top blocking them.
pub fn targeted<'a, I>(pos: (f32, f32), transforms: I) -> HashSet<Entity>
where
    I: Iterator<Item = (&'a Entity, &'a UiTransform, Option<&'a Interactable>)> + 'a,
{
    let mut entity_transforms: Vec<(&Entity, &UiTransform)> = transforms
        .filter(|(_e, t, _m)| (t.opaque || t.transparent_target) && t.position_inside(pos.0, pos.1))
        .map(|(e, t, _m)| (e, t))
        .collect();
    entity_transforms.sort_by(|(_, t1), (_, t2)| {
        t2.global_z
            .partial_cmp(&t1.global_z)
            .expect("Unexpected NaN")
    });

    let first_opaque = entity_transforms.iter().position(|(_e, t)| t.opaque);
    if let Some(i) = first_opaque {
        entity_transforms.truncate(i + 1);
    }

    entity_transforms.into_iter().map(|(e, _t)| *e).collect()
}

/// Checks if an interactable entity is at the position `pos`, doesn't have anything on top blocking
/// the check, and is below specified height.
pub fn targeted_below<'a, I>(pos: (f32, f32), height: f32, transforms: I) -> Option<Entity>
where
    I: Iterator<Item = (&'a Entity, &'a UiTransform, Option<&'a Interactable>)> + 'a,
{
    transforms
        .filter(|(_e, t, _m)| t.opaque && t.position_inside(pos.0, pos.1) && t.global_z < height)
        .max_by(|(_e1, t1, _m1), (_e2, t2, _m2)| {
            t1.global_z
                .partial_cmp(&t2.global_z)
                .expect("Unexpected NaN")
        })
        .and_then(|(e, _, m)| m.map(|_m| *e))
}
