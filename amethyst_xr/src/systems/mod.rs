use amethyst_core::specs::prelude::*;
use amethyst_core::frame_limiter::FrameLimiter;
use amethyst_core::components::Transform;
use amethyst_core::shrev::EventChannel;

use ::XRBackend;
use ::components::TrackingDevice;

pub struct XRSystem {
    pub(crate) backend: Box<dyn XRBackend>
}

impl<'a> System<'a> for XRSystem {
    type SystemData = (
        Write<'a, EventChannel<::XREvent>>,
        WriteStorage<'a, TrackingDevice>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (mut events, mut trackers, mut transforms): Self::SystemData) {
        self.backend.wait();

        if let Some(new_trackers) = self.backend.get_new_trackers() {
            events.iter_write(new_trackers.iter().map(|id| {
                ::XREvent::TrackerAdded(TrackingDevice::new(*id))
            }));
        }

        if let Some(removed_trackers) = self.backend.get_removed_trackers() {
            events.iter_write(removed_trackers.iter().map(|id| {
                ::XREvent::TrackerRemoved(*id)
            }));
        }

        for (tracker, transform) in (&mut trackers, &mut transforms).join() {
            let tracker_position_data = self.backend.get_tracker_position(tracker.id());

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
    }
}
