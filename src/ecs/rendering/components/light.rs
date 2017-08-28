//! Light resource handling.

use ecs::{Component, VecStorage};
use renderer::prelude::Light;

/// Wraps `Light` into component
pub struct LightComponent(pub Light);

impl Component for LightComponent {
    type Storage = VecStorage<Self>;
}
