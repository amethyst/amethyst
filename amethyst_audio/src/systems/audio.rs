use std::{
    iter::Iterator,
    mem::replace,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use rodio::SpatialSink;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use amethyst_core::{
    specs::prelude::{Entities, Entity, Join, Read, ReadStorage, System, WriteStorage},
    transform::GlobalTransform,
};

use crate::{
    components::{AudioEmitter, AudioListener},
    end_signal::EndSignalSource,
};

/// Syncs 3D transform data with the audio engine to provide 3D audio.
#[derive(Default)]
pub struct AudioSystem;

impl AudioSystem {
    /// Produces a new AudioSystem that uses the given listener.
    pub fn new() -> AudioSystem {
        Default::default()
    }
}

/// Add this structure to world as a resource with ID 0 to select an entity whose AudioListener
/// component will be used.  If this resource isn't found then the system will arbitrarily select
/// the first AudioListener it finds.
pub struct SelectedListener(pub Entity);

impl<'a> System<'a> for AudioSystem {
    type SystemData = (
        Option<Read<'a, SelectedListener>>,
        Entities<'a>,
        ReadStorage<'a, GlobalTransform>,
        ReadStorage<'a, AudioListener>,
        WriteStorage<'a, AudioEmitter>,
    );

    fn run(
        &mut self,
        (select_listener, entities, transform, listener, mut audio_emitter): Self::SystemData,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("audio_system");
        // Process emitters and listener.
        if let Some((listener, entity)) = select_listener
            .as_ref()
            .and_then(|sl| listener.get(sl.0).map(|l| (l, sl.0)))
            .or_else(|| (&listener, &*entities).join().next())
        {
            if let Some(listener_transform) = select_listener
                .as_ref()
                .and_then(|sl| transform.get(sl.0))
                .or_else(|| transform.get(entity))
            {
                let listener_transform = listener_transform.0;
                let left_ear_position = listener_transform
                    .transform_point(&listener.left_ear)
                    .to_homogeneous()
                    .xyz();
                let right_ear_position = listener_transform
                    .transform_point(&listener.right_ear)
                    .to_homogeneous()
                    .xyz();
                for (transform, mut audio_emitter) in (&transform, &mut audio_emitter).join() {
                    let x = transform.0[(0, 3)];
                    let y = transform.0[(1, 3)];
                    let z = transform.0[(2, 3)];
                    let emitter_position = [x, y, z];
                    // Remove all sinks whose sounds have ended.
                    audio_emitter.sinks.retain(|s| !s.1.load(Ordering::Relaxed));
                    for &mut (ref mut sink, _) in &mut audio_emitter.sinks {
                        sink.set_emitter_position(emitter_position);
                        sink.set_left_ear_position(left_ear_position.into());
                        sink.set_right_ear_position(right_ear_position.into());
                    }
                    if audio_emitter.sinks.is_empty() {
                        if let Some(mut picker) = replace(&mut audio_emitter.picker, None) {
                            if picker(&mut audio_emitter) {
                                audio_emitter.picker = Some(picker);
                            }
                        }
                    }
                    while let Some(source) = audio_emitter.sound_queue.pop() {
                        let sink = SpatialSink::new(
                            &listener.output.device,
                            emitter_position,
                            left_ear_position.into(),
                            right_ear_position.into(),
                        );
                        let atomic_bool = Arc::new(AtomicBool::new(false));
                        let clone = atomic_bool.clone();
                        sink.append(EndSignalSource::new(source, move || {
                            clone.store(true, Ordering::Relaxed);
                        }));
                        audio_emitter.sinks.push((sink, atomic_bool));
                    }
                }
            }
        }
    }
}
