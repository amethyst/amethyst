//! Code for component management.

use super::entity::Entity;

use std::any::Any;

/// A boxed component and the entity it belongs to.
#[derive(Debug)]
pub struct Component {
    pub data: Box<Any>,
    pub owner: Entity,
}

impl Component {
    pub fn new<T: Any>(owner: Entity, data: T) -> Component {
        Component {
            data: Box::new(data),
            owner: owner,
        }
    }
}
