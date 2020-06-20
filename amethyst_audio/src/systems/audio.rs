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
    ecs::prelude::*,
    math::convert,
    transform::LocalToWorld,
};

use crate::{
    components::{AudioEmitter, AudioListener},
    end_signal::EndSignalSource,
    output::Output,
};

/// Syncs 3D transform data with the audio engine to provide 3D audio.
#[derive(Debug, Default, new)]
pub struct AudioSystem(Output);

/// Add this structure to world as a resource with ID 0 to select an entity whose AudioListener
/// component will be used.  If this resource isn't found then the system will arbitrarily select
/// the first AudioListener it finds.
#[derive(Debug)]
pub struct SelectedListener(pub Entity);

/// Creates a new audio system.
pub fn build_audio_system(_world: &mut World, _res: &mut Resources) -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("AudioSystem")
        .read_resource::<Option<Output>>()
        .read_resource::<Option<SelectedListener>>()
        .with_query(<Read<AudioListener>>::query())
        .with_query(<(Write<AudioEmitter>, Read<LocalToWorld>)>::query())
        .build(move |_commands, world, (output, select_listener), (q_audio_listener, q_audio_emitter)| {
            #[cfg(feature = "profiler")]
            profile_scope!("audio_system");
            // Process emitters and listener.
            if let Some((entity, listener)) = select_listener
                .as_ref()
                .and_then(|sl| world.get_component::<AudioListener>(sl.0).map(|c| (sl.0, (*c).clone())))
                .or_else(|| q_audio_listener.iter_entities(world).next().map(|(e,c)| (e,(*c).clone())))
            {
                if let Some(listener_transform) = select_listener
                    .as_ref()
                    .and_then(|sl| world.get_component::<LocalToWorld>(sl.0).map(|c| (*c).clone()))
                    .or_else(|| world.get_component::<LocalToWorld>(entity).map(|c| (*c).clone()))
                {
                    let listener_transform = listener_transform.0;
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
                    q_audio_emitter.for_each_mut(world, |(mut audio_emitter, transform)| {
                        let emitter_position: [f32; 3] = {
                            let x = transform.0[(0, 3)];
                            let y = transform.0[(1, 3)];
                            let z = transform.0[(2, 3)];
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
                            if let Some(output) = &**output {
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
                    });
                }
            }
        })
}
