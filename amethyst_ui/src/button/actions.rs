use amethyst_core::ecs::prelude::Entity;
use amethyst_assets::Handle;
use amethyst_rendy::types::Texture;

/// Describes an action targeted at a `UiButton`.
#[derive(Debug, Clone)]
pub struct UiButtonAction<B: Backend> {
    /// The target entity for the action
    pub target: Entity,
    /// The event type of the action
    pub event_type: UiButtonActionType,
}

/// Describes the type of a `UiButtonAction`.
#[derive(Debug, Clone)]
pub enum UiButtonActionType<B: Backend> {
    /// Sets the texture of a `UiButton` to the given `TextureHandle`.
    SetTexture(Handle<Texture<B>>),
    /// Removes a previously set `TextureHandle` on a `UiButton`.
    UnsetTexture(Handle<Texture<B>>),

    // TODO(happens): Replace this with `palette` types
    /// Sets the text color of the primary text child of a `UiButton`.
    SetTextColor([f32; 4]),
    /// Removes a previously set color from the primary text child
    /// of a `UiButton`.
    UnsetTextColor([f32; 4]),
}
