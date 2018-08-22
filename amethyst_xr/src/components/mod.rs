use amethyst_core::specs::storage::BTreeStorage;
use amethyst_core::specs::Component;

use amethyst_renderer::{Mesh, MeshHandle, TextureHandle};

#[derive(Clone)]
pub struct TrackingDevice {
    id: u32,
    haptic_duration: Option<u16>,
    mesh: Option<MeshHandle>,
    texture: Option<TextureHandle>,
    tracking: bool,
    available: bool,

    pub(crate) is_camera: bool,

    pub(crate) has_render_model: bool,
    pub(crate) render_model_enabled: bool,
}

impl TrackingDevice {
    pub(crate) fn new(id: u32) -> TrackingDevice {
        TrackingDevice {
            id,
            mesh: None,
            texture: None,
            tracking: false,
            available: false,
            haptic_duration: None,

            is_camera: false,

            has_render_model: false,
            render_model_enabled: false,
        }
    }

    pub(crate) fn set_mesh(&mut self, mesh: Option<MeshHandle>) {
        self.mesh = mesh;
    }

    pub(crate) fn set_texture(&mut self, texture: Option<TextureHandle>) {
        self.texture = texture
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

    pub fn has_mesh(&self) -> bool {
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
    type Storage = BTreeStorage<Self>;
}
