use amethyst_input::InputHandler;
use amethyst_renderer::MouseButton;
use shrev::EventChannel;
use specs::{Component, Entities, Entity, Fetch, FetchMut, Join, NullStorage, ReadStorage, System};
use transform::UiTransform;

/// The type of ui event.
/// Click happens if you start and stop clicking on the same ui element.
#[derive(Debug)]
pub enum UiEventType {
    Click,
    ClickStart,
    ClickStop,
    HoverStart,
    HoverStop,
}

/// A ui event instance.
#[derive(Debug)]
pub struct UiEvent {
    pub event_type: UiEventType,
    pub target: Entity,
}

impl UiEvent {
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
pub struct UiMouseSystem {
    was_down: bool,
    old_pos: (f32, f32),
    click_started_on: Option<Entity>,
}

impl UiMouseSystem {
    pub fn new() -> Self {
        UiMouseSystem {
            was_down: false,
            old_pos: (0.0, 0.0),
            click_started_on: None,
        }
    }

    fn pos_in_rect(
        &self,
        pos_x: f32,
        pos_y: f32,
        rect_x: f32,
        rect_y: f32,
        width: f32,
        height: f32,
    ) -> bool {
        pos_x > rect_x && pos_x < rect_x + width && pos_y > rect_y && pos_y < rect_y + height
    }
}

impl<'a> System<'a> for UiMouseSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, UiTransform>,
        ReadStorage<'a, MouseReactive>,
        Fetch<'a, InputHandler<String, String>>,
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
                let is_in_rect = self.pos_in_rect(x, y, tr.x, tr.y, tr.width, tr.height);
                let was_in_rect = self.pos_in_rect(
                    self.old_pos.0,
                    self.old_pos.1,
                    tr.x,
                    tr.y,
                    tr.width,
                    tr.height,
                );

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
