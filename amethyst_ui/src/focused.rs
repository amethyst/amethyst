use specs::Entity;

/// This resource stores the currently focused UI element.
pub struct UiFocused {
    /// The entity containing the focused UI element.
    pub entity: Option<Entity>,
}
