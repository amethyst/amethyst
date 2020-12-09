use shrinkwraprs::Shrinkwrap;

use crate::ecs::*;

#[derive(Shrinkwrap, Debug, Copy, Clone, Eq, PartialEq)]
#[shrinkwrap(mutable)]
/// Component used for hierarchy definition.
/// Parent entity will automatically get [Children] component.
pub struct Parent(pub Entity);

#[derive(Shrinkwrap, Debug, Copy, Clone, Eq, PartialEq)]
#[shrinkwrap(mutable)]
/// Temporary component used for observing hierarchy changes.
pub struct PreviousParent(pub Option<Entity>);
