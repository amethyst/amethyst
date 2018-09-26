use std::sync::Mutex;

use amethyst_core::components::Transform;
use amethyst_core::frame_limiter::FrameLimiter;
use amethyst_core::shrev::EventChannel;
use amethyst_core::specs::prelude::*;

use components::TrackingDevice;
use {XRBackend, XRInfo};

pub struct XRSystem {
    pub(crate) backend: Option<Box<dyn XRBackend>>,
}

impl<'a> System<'a> for XRSystem {
    type SystemData = (
        WriteExpect<'a, XRInfo>,
        Write<'a, EventChannel<::XREvent>>,
        WriteStorage<'a, TrackingDevice>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (mut info, mut events, mut trackers, mut transforms): Self::SystemData) {
        let mut backend = info.backend();

        backend.wait();

        if let Some(new_trackers) = backend.get_new_trackers() {
            events.iter_write(new_trackers.into_iter().map(|(id, capabilities)| {
                let mut tracker = TrackingDevice::new(id, capabilities);
                ::XREvent::TrackerAdded(tracker)
            }));
        }

        if let Some(removed_trackers) = backend.get_removed_trackers() {
            events.iter_write(
                removed_trackers
                    .iter()
                    .map(|id| ::XREvent::TrackerRemoved(*id)),
            );
        }

        for (tracker, transform) in (&mut trackers, &mut transforms).join() {
            // Set position and rotation
            let tracker_position_data = backend.get_tracker_position(tracker.id());

            transform.translation = tracker_position_data.position;
            transform.rotation = tracker_position_data.rotation;
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        if let Some(frame_limiter) = res.get_mut::<FrameLimiter>() {
            frame_limiter.set_rate(Default::default(), 0);
        }

        res.insert(EventChannel::<::XREvent>::new());

        let targets = self
            .backend
            .as_mut()
            .unwrap()
            .get_gl_target_info(0.01, 1000.0);

        res.insert(XRInfo {
            targets,

            defined_area: vec![],
            backend: Mutex::new(self.backend.take().unwrap()),
        });
    }
}
