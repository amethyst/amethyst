use std::{hash::Hash, marker::PhantomData};

use amethyst_core::{
    shrev::EventChannel,
    specs::prelude::{
        Component, Entities, Entity, Join, Read, ReadExpect, ReadStorage, System, Write,
    },
    specs::storage::NullStorage,
};
use amethyst_input::InputHandler;
use amethyst_renderer::{MouseButton, ScreenDimensions};

use crate::transform::UiTransform;

/// The type of ui event.
/// Click happens if you start and stop clicking on the same ui element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiEventType {
    /// When an element is clicked normally.
    Click,
    /// When the element starts being clicked (On left mouse down).
    ClickStart,
    /// When the element stops being clicked (On left mouse up).
    ClickStop,
    /// When the cursor gets over an element.
    HoverStart,
    /// When the cursor stops being over an element.
    HoverStop,
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

/// A component that tags an entity as reactive to ui events.
/// Will only work if the entity has a UiTransform component attached to it.
/// Without this, the ui element will not generate events.
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct MouseReactive;

impl Component for MouseReactive {
    type Storage = NullStorage<MouseReactive>;
}

/// The system that generates events for `MouseReactive` enabled entities.
/// The generic types A and B represent the A and B generic parameter of the InputHandler<A,B>.
pub struct UiMouseSystem<A, B> {
    was_down: bool,
    click_started_on: Option<Entity>,
    last_target: Option<Entity>,
    _marker: PhantomData<(A, B)>,
}

impl<A, B> UiMouseSystem<A, B> {
    /// Creates a new UiMouseSystem.
    pub fn new() -> Self {
        UiMouseSystem {
            was_down: false,
            click_started_on: None,
            last_target: None,
            _marker: PhantomData,
        }
    }
}

impl<'a, A, B> System<'a> for UiMouseSystem<A, B>
where
    A: Send + Sync + Eq + Hash + Clone + 'static,
    B: Send + Sync + Eq + Hash + Clone + 'static,
{
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, UiTransform>,
        ReadStorage<'a, MouseReactive>,
        Read<'a, InputHandler<A, B>>,
        ReadExpect<'a, ScreenDimensions>,
        Write<'a, EventChannel<UiEvent>>,
    );

    fn run(
        &mut self,
        (entities, transform, react, input, screen_dimensions, mut events): Self::SystemData,
    ) {
        let down = input.mouse_button_is_down(MouseButton::Left);

        // TODO: To replace on InputHandler generate OnMouseDown and OnMouseUp events
        let click_started = down && !self.was_down;
        let click_stopped = !down && self.was_down;

        if let Some((pos_x, pos_y)) = input.mouse_position() {
            let x = pos_x as f32;
            let y = screen_dimensions.height() - pos_y as f32;

            let target = targeted((x, y), (&*entities, &transform, react.maybe()).join());
            if target != self.last_target {
                if let Some(last_target) = self.last_target {
                    events.single_write(UiEvent::new(UiEventType::HoverStop, last_target));
                }
                if let Some(target) = target {
                    events.single_write(UiEvent::new(UiEventType::HoverStart, target));
                }
            }

            if let Some(e) = target {
                if click_started {
                    events.single_write(UiEvent::new(UiEventType::ClickStart, e));
                    self.click_started_on = Some(e);
                } else if click_stopped {
                    if let Some(e2) = self.click_started_on {
                        if e2 == e {
                            events.single_write(UiEvent::new(UiEventType::Click, e2));
                        }
                    }
                }
            }

            self.last_target = target;
        }

        // Could be used for drag and drop
        if click_stopped {
            if let Some(e) = self.click_started_on {
                events.single_write(UiEvent::new(UiEventType::ClickStop, e));
                self.click_started_on = None;
            }
        }

        self.was_down = down;
    }
}

fn targeted<'a, I>(pos: (f32, f32), transforms: I) -> Option<Entity>
where
    I: Iterator<Item = (Entity, &'a UiTransform, Option<&'a MouseReactive>)> + 'a,
{
    transforms
        .filter(|(_e, t, _m)| t.opaque && t.position_inside(pos.0, pos.1))
        .max_by(|(_e1, t1, _m1), (_e2, t2, _m2)| {
            t1.global_z
                .partial_cmp(&t2.global_z)
                .expect("Unexpected NaN")
        })
        .and_then(|(e, _, m)| m.map(|_m| e))
}
