use std::{
    iter::Iterator,
    mem::replace,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use derive_new::new;
use rodio::SpatialSink;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use amethyst_core::{
    ecs::prelude::{
        Entities, Entity, Join, Read, ReadStorage, System, SystemData, World, WriteStorage,
    },
    math::convert,
    transform::Transform,
    SystemDesc,
};

use crate::{
    components::{AudioEmitter, AudioListener},
    end_signal::EndSignalSource,
    output::Output,
};

/// Builds an `AudioSystem`.
#[derive(Default, Debug, new)]
pub struct AudioSystemDesc {
    /// Audio `Output`.
    pub output: Output,
}

impl<'a, 'b> SystemDesc<'a, 'b, AudioSystem> for AudioSystemDesc {
    fn build(self, world: &mut World) -> AudioSystem {
        <AudioSystem as System<'_>>::SystemData::setup(world);

        world.insert(self.output.clone());

        AudioSystem::new(self.output)
    }
}

/// Syncs 3D transform data with the audio engine to provide 3D audio.
#[derive(Debug, Default, new)]
pub struct AudioSystem(Output);

/// Add this structure to world as a resource with ID 0 to select an entity whose AudioListener
/// component will be used.  If this resource isn't found then the system will arbitrarily select
/// the first AudioListener it finds.
#[derive(Debug)]
pub struct SelectedListener(pub Entity);

impl<'a> System<'a> for AudioSystem {
    type SystemData = (
        Option<Read<'a, Output>>,
        Option<Read<'a, SelectedListener>>,
        Entities<'a>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, AudioListener>,
        WriteStorage<'a, AudioEmitter>,
    );

    fn run(
        &mut self,
        (output, select_listener, entities, transform, listener, mut audio_emitter): Self::SystemData,
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
                let listener_transform = listener_transform.global_matrix();
                let left_ear_position: [f32; 3] = {
                    let pos = listener_transform
                        .transform_point(&listener.left_ear)
                        .to_homogeneous()
                        .xyz();
                    [convert(pos.x), convert(pos.y), convert(pos.z)]
                };
                let right_ear_position: [f32; 3] = {
                    let pos = listener_transform
                        .transform_point(&listener.right_ear)
                        .to_homogeneous()
                        .xyz();
                    [convert(pos.x), convert(pos.y), convert(pos.z)]
                };
                for (transform, mut audio_emitter) in (&transform, &mut audio_emitter).join() {
                    let emitter_position: [f32; 3] = {
                        let x = transform.global_matrix()[(0, 3)];
                        let y = transform.global_matrix()[(1, 3)];
                        let z = transform.global_matrix()[(2, 3)];
                        [convert(x), convert(y), convert(z)]
                    };
                    // Remove all sinks whose sounds have ended.
                    audio_emitter.sinks.retain(|s| !s.1.load(Ordering::Relaxed));
                    for &mut (ref mut sink, _) in &mut audio_emitter.sinks {
                        sink.set_emitter_position(emitter_position);
                        sink.set_left_ear_position(left_ear_position);
                        sink.set_right_ear_position(right_ear_position);
                    }
                    if audio_emitter.sinks.is_empty() {
                        if let Some(mut picker) = replace(&mut audio_emitter.picker, None) {
                            if picker(&mut audio_emitter) {
                                audio_emitter.picker = Some(picker);
                            }
                        }
                    }
                    while let Some(source) = audio_emitter.sound_queue.pop() {
                        if let Some(output) = &output {
                            let sink = SpatialSink::new(
                                &output.device,
                                emitter_position,
                                left_ear_position,
                                right_ear_position,
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
}
