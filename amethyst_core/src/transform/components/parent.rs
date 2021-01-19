use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use type_uuid::TypeUuid;

use crate::ecs::*;

/// Component used for hierarchy definition.
/// Parent entity will automatically get [Children] component.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, SerdeDiff, TypeUuid)]
#[uuid = "4f1cb751-d650-4cf0-8418-63cff1e85c84"]
pub struct Parent(#[serde_diff(opaque)] pub Entity);

impl Default for Parent {
    fn default() -> Self {
        unimplemented!()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
/// Temporary component used for observing hierarchy changes.
pub struct PreviousParent(pub Option<Entity>);
