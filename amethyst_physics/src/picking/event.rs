///! Adapted from amethyst_ui/src/event.rs
use std::hash::Hash;
use std::marker::PhantomData;

use amethyst_core::shrev::EventChannel;
use amethyst_core::specs::prelude::*;
use amethyst_input::InputHandler;
use amethyst_renderer::MouseButton;
use amethyst_ui::{UiEvent, UiEventType};

use super::pick::Picked;

/// The system that generates `UiEvents` for `Pickable` entities, so these entities can be utilized by systems that deal with the UI.
/// This is a counterpart for `UiMouseSystem` and emits events to the same channel.
/// The generic types A and B represent the A and B generic parameter of the InputHandler<A,B>.
pub struct PickEventSys<A, B> {
    was_down: bool,
    click_started_on: Option<Entity>,
    last_target: Option<Entity>,
    _marker: PhantomData<(A, B)>,
}

impl<A, B> PickEventSys<A, B> {
    /// Initialize a new `PickEventSys`.
    pub fn new() -> Self {
        PickEventSys {
            was_down: false,
            click_started_on: None,
            last_target: None,
            _marker: PhantomData,
        }
    }
}

impl<'a, A, B> System<'a> for PickEventSys<A, B>
where
    A: Send + Sync + Eq + Hash + Clone + 'static,
    B: Send + Sync + Eq + Hash + Clone + 'static,
{
    type SystemData = (
        Read<'a, InputHandler<A, B>>,
        Read<'a, Picked>,
        Write<'a, EventChannel<UiEvent>>,
    );

    fn run(&mut self, (input, picked, mut events): Self::SystemData) {
        let down = input.mouse_button_is_down(MouseButton::Left);

        let click_started = down && !self.was_down;
        let click_stopped = !down && self.was_down;

        let target = picked.entity_intersection.map(|(entity, _)| entity);

        // XXX: The rest of this function is the same as amethyst_ui/src/event.rs

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
