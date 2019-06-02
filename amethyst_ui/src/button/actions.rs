use crate::UiImageComponent;
use amethyst_core::ecs::prelude::Entity;

/// Describes an action targeted at a `UiButton`.
#[derive(Debug, Clone)]
pub struct UiButtonAction {
    /// The target entity for the action
    pub target: Entity,
    /// The event type of the action
    pub event_type: UiButtonActionType,
}

/// Describes the type of a `UiButtonAction`.
#[derive(Debug, Clone)]
pub enum UiButtonActionType {
    /// Sets the texture of a `UiButton` to the given `UiImage`.
    SetImage(UiImageComponent),
    /// Removes a previously set `UiImage` on a `UiButton`.
    UnsetTexture(UiImageComponent),
    /// Sets the text color of the primary text child of a `UiButton`.
    SetTextColor([f32; 4]),
    /// Removes a previously set color from the primary text child
    /// of a `UiButton`.
    UnsetTextColor([f32; 4]),
}
