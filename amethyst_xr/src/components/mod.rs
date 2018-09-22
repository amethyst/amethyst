use amethyst_core::specs::storage::HashMapStorage;
use amethyst_core::specs::Component;

use TrackerCapabilities;

#[derive(Clone)]
pub struct TrackingDevice {
    id: u32,
    haptic_duration: Option<u16>,
    tracking: bool,
    available: bool,

    capabilities: TrackerCapabilities,
}

impl TrackingDevice {
    pub(crate) fn new(id: u32, capabilities: TrackerCapabilities) -> TrackingDevice {
        TrackingDevice {
            id,
            tracking: false,
            available: false,
            haptic_duration: None,

            capabilities,
        }
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

    pub fn tracking(&self) -> bool {
        self.tracking
    }

    pub fn available(&self) -> bool {
        self.available
    }

    pub fn trigger_haptics(&mut self, duration: u16) {
        self.haptic_duration = Some(duration);
    }

    pub fn capabilities(&self) -> &TrackerCapabilities {
        &self.capabilities
    }
}

impl Component for TrackingDevice {
    type Storage = HashMapStorage<Self>;
}

pub struct RenderModelComponent(String);

impl Component for RenderModelComponent {
    type Storage = HashMapStorage<Self>;
}
