use amethyst_core::ecs::prelude::{Component, DenseVecStorage};

use crate::{
    event::{UiEvent, UiEventType},
    event_retrigger::{EventRetrigger, EventRetriggerSystem, EventRetriggerSystemDesc},
    EventReceiver, UiButtonAction,
};

/// Builds a `UiButtonActionRetriggerSystem`.
pub type UiButtonActionRetriggerSystemDesc = EventRetriggerSystemDesc<UiButtonActionRetrigger>;

/// Provides an `EventRetriggerSystem` that will handle incoming `UiEvents`
/// and trigger `UiButtonAction`s for `UiButton`s with an attached
/// `UiButtonActionRetrigger` component.
pub type UiButtonActionRetriggerSystem = EventRetriggerSystem<UiButtonActionRetrigger>;

/// Attach this to an entity with a `UiButton` attached to it to
/// trigger specific events when a user interaction happens.
#[derive(Debug, Default, Clone)]
pub struct UiButtonActionRetrigger {
    /// The `UiButtonAction`s that should happen when the user begins a click
    /// on the `UiButton`
    pub on_click_start: Vec<UiButtonAction>,
    /// The `UiButtonAction`s that should happen when the user ends a click
    /// on the `UiButton`
    pub on_click_stop: Vec<UiButtonAction>,
    /// The `UiButtonAction`s that should happen when the user start hovering
    /// over the `UiButton`
    pub on_hover_start: Vec<UiButtonAction>,
    /// The `UiButtonAction`s that should happen when the user stops hovering
    /// over the `UiButton`
    pub on_hover_stop: Vec<UiButtonAction>,
}

impl Component for UiButtonActionRetrigger {
    type Storage = DenseVecStorage<Self>;
}

impl EventRetrigger for UiButtonActionRetrigger {
    type In = UiEvent;
    type Out = UiButtonAction;

    fn apply<R>(&self, event: &Self::In, out: &mut R)
    where
        R: EventReceiver<Self::Out>,
    {
        match event.event_type {
            UiEventType::ClickStart => out.receive(&self.on_click_start),
            UiEventType::ClickStop => out.receive(&self.on_click_stop),
            UiEventType::HoverStart => out.receive(&self.on_hover_start),
            UiEventType::HoverStop => out.receive(&self.on_hover_stop),
            _ => {}
        };
    }
}
