use std::{
    iter::Iterator,
    mem::replace,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use amethyst_core::{ecs::*, math::convert, transform::Transform};
use rodio::SpatialSink;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{
    components::{AudioEmitter, AudioListener},
    end_signal::EndSignalSource,
    output::OutputWrapper,
};

/// Syncs 3D transform data with the audio engine to provide 3D audio.
#[derive(Debug)]
pub struct AudioSystem;

/// Add this structure to world as a resource with ID 0 to select an entity whose AudioListener
/// component will be used.  If this resource isn't found then the system will arbitrarily select
/// the first AudioListener it finds.
#[derive(Debug, Default)]
pub struct SelectedListener(pub Option<Entity>);

impl System for AudioSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("AudioSystem")
                .read_resource::<OutputWrapper>()
                .read_resource::<SelectedListener>()
                .with_query(<(Entity, Read<AudioListener>)>::query())
                .with_query(<(Write<AudioEmitter>, Read<Transform>)>::query())
                .build(
                    move |_commands,
                          world,
                          (wrapper, select_listener),
                          (q_audio_listener, q_audio_emitter)| {
                        #[cfg(feature = "profiler")]
                        profile_scope!("audio_system");
                        // Process emitters and listener.
                        if let Some((entity, listener)) = if let Some(entity) = select_listener.0 {
                            // Find entity refered by SelectedListener resource
                            world
                                .entry_ref(entity)
                                .ok()
                                .and_then(|entry| entry.into_component::<AudioListener>().ok())
                                .map(|audio_listener| (entity, audio_listener))
                        } else {
                            // Otherwise, select the first available AudioListener
                            q_audio_listener
                                .iter(world)
                                .next()
                                .map(|(entity, audio_listener)| (*entity, audio_listener))
                        } {
                            if let Some(listener_transform) = world
                                .entry_ref(entity)
                                .ok()
                                .and_then(|entry| entry.into_component::<Transform>().ok())
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
                                q_audio_emitter.for_each_mut(
                                    world,
                                    |(mut audio_emitter, transform)| {
                                        let emitter_position: [f32; 3] = {
                                            let x = transform.global_matrix()[(0, 3)];
                                            let y = transform.global_matrix()[(1, 3)];
                                            let z = transform.global_matrix()[(2, 3)];
                                            [convert(x), convert(y), convert(z)]
                                        };
                                        // Remove all sinks whose sounds have ended.
                                        audio_emitter
                                            .sinks
                                            .retain(|s| !s.1.load(Ordering::Relaxed));
                                        for &mut (ref mut sink, _) in &mut audio_emitter.sinks {
                                            sink.set_emitter_position(emitter_position);
                                            sink.set_left_ear_position(left_ear_position);
                                            sink.set_right_ear_position(right_ear_position);
                                        }
                                        if audio_emitter.sinks.is_empty() {
                                            if let Some(mut picker) =
                                                replace(&mut audio_emitter.picker, None)
                                            {
                                                if picker(&mut audio_emitter) {
                                                    audio_emitter.picker = Some(picker);
                                                }
                                            }
                                        }
                                        while let Some(source) = audio_emitter.sound_queue.pop() {
                                            if let Some(output) = &wrapper.output {
                                                let sink = SpatialSink::new(
                                                    &output.device,
                                                    emitter_position,
                                                    left_ear_position,
                                                    right_ear_position,
                                                );
                                                let atomic_bool = Arc::new(AtomicBool::new(false));
                                                let clone = atomic_bool.clone();
                                                sink.append(EndSignalSource::new(
                                                    source,
                                                    move || {
                                                        clone.store(true, Ordering::Relaxed);
                                                    },
                                                ));
                                                audio_emitter.sinks.push((sink, atomic_bool));
                                            }
                                        }
                                    },
                                );
                            }
                        }
                    },
                ),
        )
    }
}
