//! Light resource handling.

use renderer::prelude::Light;

use ecs::{Component, VecStorage};

/// Wraps `Light` into component
pub struct LightComponent(pub Light);

impl Component for LightComponent {
    type Storage = VecStorage<Self>;
}
