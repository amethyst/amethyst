use amethyst_core::specs::storage::HashMapStorage;
use amethyst_core::specs::Component;

use amethyst_renderer::{Mesh, MeshHandle, TextureHandle};

pub type ComponentModel = (String, MeshHandle, TextureHandle);

#[derive(Clone)]
pub struct TrackingDevice {
    id: u32,
    haptic_duration: Option<u16>,
    tracking: bool,
    available: bool,

    pub(crate) component_models: Vec<ComponentModel>,

    pub(crate) is_camera: bool,

    pub(crate) has_render_model: bool,
    pub(crate) render_model_enabled: bool,
}

impl TrackingDevice {
    pub(crate) fn new(id: u32) -> TrackingDevice {
        TrackingDevice {
            id,
            tracking: false,
            available: false,
            haptic_duration: None,

            component_models: Vec::new(),

            is_camera: false,

            has_render_model: false,
            render_model_enabled: false,
        }
    }

    pub fn component_models(&self) -> &[ComponentModel] {
        &self.component_models
    }

    pub(crate) fn set_tracking(&mut self, tracking: bool) {
        self.tracking = tracking;
    }

    pub(crate) fn set_available(&mut self, available: bool) {
        self.available = available
    }

    pub fn id(&self) -> u32 {
        return self.id;
    }

    pub fn has_model(&self) -> bool {
        self.mesh.is_some()
    }

    pub fn has_texture(&self) -> bool {
        self.texture.is_some()
    }

    pub fn mesh(&self) -> Option<MeshHandle> {
        self.mesh.clone()
    }

    pub fn texture(&self) -> Option<TextureHandle> {
        self.texture.clone()
    }

    pub fn has_model(&self) -> bool {
        self.has_render_model
    }

    pub fn set_render_model_enabled(&mut self, enabled: bool) {
        self.render_model_enabled = enabled;
    }

    pub fn render_model_enabled(&self) -> bool {
        self.render_model_enabled
    }

    pub fn tracking(&self) -> bool {
        self.tracking
    }

    pub fn available(&self) -> bool {
        self.available
    }

    pub fn trigger_haptics(&mut self, duration: u16) {
        self.haptic_duration = Some(duration);
    }

    pub fn is_camera(&self) -> bool {
        self.is_camera
    }
}

impl Component for TrackingDevice {
    type Storage = HashMapStorage<Self>;
}

pub struct RenderModelComponent(String);

impl Component for RenderModelComponent {
    type Storage = HashMapStorage<Self>;
}
