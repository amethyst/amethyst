use crate::ecs::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
/// Component used for hierarchy definition.
/// Parent entity will automatically get [Children] component.
pub struct Parent(pub Entity);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
/// Temporary component used for observing hierarchy changes.
pub struct PreviousParent(pub Option<Entity>);
