use amethyst_core::ecs::prelude::{Component, DenseVecStorage};

use crate::{
    render::UiRenderer,
    event::{UiEvent, UiEventType},
    event_retrigger::{EventRetrigger, EventRetriggerSystem},
    EventReceiver, UiButtonAction,
};

/// Provides an `EventRetriggerSystem` that will handle incoming `UiEvents`
/// and trigger `UiButtonAction`s for `UiButton`s with an attached
/// `UiButtonActionRetrigger` component.
pub type UiButtonActionRetriggerSystem<R: UiRenderer> =
    EventRetriggerSystem<UiButtonActionRetrigger<R>>;

/// Attach this to an entity with a `UiButton` attached to it to
/// trigger specific events when a user interaction happens.
#[derive(Debug)]
pub struct UiButtonActionRetrigger<R: UiRenderer> {
    /// The `UiButtonAction`s that should happen when the user begins a click
    /// on the `UiButton`
    pub on_click_start: Vec<UiButtonAction<R>>,
    /// The `UiButtonAction`s that should happen when the user ends a click
    /// on the `UiButton`
    pub on_click_stop: Vec<UiButtonAction<R>>,
    /// The `UiButtonAction`s that should happen when the user start hovering
    /// over the `UiButton`
    pub on_hover_start: Vec<UiButtonAction<R>>,
    /// The `UiButtonAction`s that should happen when the user stops hovering
    /// over the `UiButton`
    pub on_hover_stop: Vec<UiButtonAction<R>>,
}

impl<R> Component for UiButtonActionRetrigger<R> where R: UiRenderer {
    type Storage = DenseVecStorage<Self>;
}

impl<R> EventRetrigger for UiButtonActionRetrigger<R> where R: UiRenderer {
    type In = UiEvent;
    type Out = UiButtonAction<R>;

    fn apply<T>(&self, event: &Self::In, out: &mut T)
    where
        T: EventReceiver<Self::Out>,
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
