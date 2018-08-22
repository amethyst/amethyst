use amethyst_assets::{AssetStorage, Loader};
use amethyst_core::components::Transform;
use amethyst_core::frame_limiter::FrameLimiter;
use amethyst_core::shrev::EventChannel;
use amethyst_core::specs::prelude::*;
use amethyst_renderer::{Mesh, MeshData, Texture};

use components::TrackingDevice;
use {TrackerModelLoadStatus, XRBackend};

pub struct XRSystem {
    pub(crate) backend: Box<dyn XRBackend>,
}

impl<'a> System<'a> for XRSystem {
    type SystemData = (
        Write<'a, EventChannel<::XREvent>>,
        WriteStorage<'a, TrackingDevice>,
        WriteStorage<'a, Transform>,
        ReadExpect<'a, Loader>,
        Read<'a, AssetStorage<Mesh>>,
        Read<'a, AssetStorage<Texture>>,
    );

    fn run(
        &mut self,
        (mut events, mut trackers, mut transforms, loader, meshes, textures): Self::SystemData,
    ) {
        self.backend.wait();

        if let Some(new_trackers) = self.backend.get_new_trackers() {
            events.iter_write(new_trackers.iter().map(|id| {
                let mut tracker = TrackingDevice::new(*id);
                tracker.has_render_model = self.backend.tracker_has_model(tracker.id());
                ::XREvent::TrackerAdded(tracker)
            }));
        }

        if let Some(removed_trackers) = self.backend.get_removed_trackers() {
            events.iter_write(
                removed_trackers
                    .iter()
                    .map(|id| ::XREvent::TrackerRemoved(*id)),
            );
        }

        for (tracker, transform) in (&mut trackers, &mut transforms).join() {
            // Set position and rotation
            let tracker_position_data = self.backend.get_tracker_position(tracker.id());

            transform.translation = tracker_position_data.position;
            transform.rotation = tracker_position_data.rotation;

            // Update render model if requested
            if tracker.render_model_enabled && tracker.mesh().is_none() {
                if let TrackerModelLoadStatus::Available((vertices, indices), maybe_texture) =
                    self.backend.get_tracker_model(tracker.id())
                {
                    let vertices = MeshData::PosNormTangTex(
                        indices
                            .iter()
                            .map(|i| vertices[*i as usize].clone())
                            .collect(),
                    );

                    let mesh = loader.load_from_data(vertices, (), &meshes);
                    tracker.set_mesh(Some(mesh));

                    if let Some(texture) = maybe_texture {
                        let texture = loader.load_from_data(texture, (), &textures);
                        tracker.set_texture(Some(texture));
                    }

                    println!("Render model loaded for tracker {}", tracker.id());
                    events.single_write(::XREvent::TrackerModelLoaded(tracker.id()))
                }
            }
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
