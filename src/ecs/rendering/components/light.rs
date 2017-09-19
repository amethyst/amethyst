//! Light resource handling.

use renderer::prelude::Light;

use ecs::{Component, VecStorage};

/// Wraps `Light` into component
#[derive(Clone, Debug)]
pub struct LightComponent(pub Light);

impl Component for LightComponent {
    type Storage = VecStorage<Self>;
}

impl AsRef<Light> for LightComponent {
    fn as_ref(&self) -> &Light {
        &self.0
    }
}