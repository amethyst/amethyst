use amethyst_core::specs::{Component, Entities, Entity, Fetch, FetchMut, Join, NullStorage,
                           ReadStorage, System};
use amethyst_input::InputHandler;
use amethyst_renderer::MouseButton;
use shrev::EventChannel;
use std::hash::Hash;
use std::marker::PhantomData;
use transform::UiTransform;

/// The type of ui event.
/// Click happens if you start and stop clicking on the same ui element.
#[derive(Debug)]
pub enum UiEventType {
    /// When an element is clicked normally.
    /// Happens when the element both start and stops being clicked.
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
#[derive(Debug)]
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
#[derive(Default)]
pub struct MouseReactive;

impl Component for MouseReactive {
    type Storage = NullStorage<MouseReactive>;
}

/// The system that generates events for `MouseReactive` enabled entities.
/// The generic types A and B represent the A and B generic parameter of the InputHandler<A,B>.
pub struct UiMouseSystem<A, B> {
    was_down: bool,
    old_pos: (f32, f32),
    click_started_on: Option<Entity>,
    _marker1: PhantomData<A>,
    _marker2: PhantomData<B>,
}

impl<A, B> UiMouseSystem<A, B> {
    /// Creates a new UiMouseSystem.
    pub fn new() -> Self {
        UiMouseSystem {
            was_down: false,
            old_pos: (0.0, 0.0),
            click_started_on: None,
            _marker1: PhantomData,
            _marker2: PhantomData,
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
        Fetch<'a, InputHandler<A, B>>,
        FetchMut<'a, EventChannel<UiEvent>>,
    );

    fn run(&mut self, (entities, transform, react, input, mut events): Self::SystemData) {
        let down = input.mouse_button_is_down(MouseButton::Left);

        // to replace on InputHandler generate OnMouseDown and OnMouseUp events
        let click_started = down && !self.was_down;
        let click_stopped = !down && self.was_down;
        if let Some((pos_x, pos_y)) = input.mouse_position() {
            let x = pos_x as f32;
            let y = pos_y as f32;
            for (tr, e, _) in (&transform, &*entities, &react).join() {
                let is_in_rect = tr.position_inside(x, y);
                let was_in_rect = tr.position_inside(self.old_pos.0, self.old_pos.1);

                if is_in_rect && !was_in_rect {
                    events.single_write(UiEvent::new(UiEventType::HoverStart, e));
                } else if !is_in_rect && was_in_rect {
                    events.single_write(UiEvent::new(UiEventType::HoverStop, e));
                }

                if is_in_rect {
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
            }

            self.old_pos = (x, y);
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
