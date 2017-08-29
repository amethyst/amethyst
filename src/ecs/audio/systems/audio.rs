use std::iter::Iterator;
use std::mem::replace;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use cgmath::{Matrix4, Transform, Point3};
use rodio::{SpatialSink, Source, Sample};

use ecs::{Fetch, Join, System, ReadStorage, WriteStorage, Entity};
use ecs::audio::components::{AudioEmitter, AudioListener};
use ecs::transform::Transform as TransformComponent;

/// Syncs 3D transform data with the audio engine to provide 3D audio.
pub struct AudioSystem;

impl AudioSystem {
    /// Produces a new AudioSystem that uses the given listener.
    pub fn new() -> AudioSystem {
        AudioSystem
    }
}

/// Add this structure to world as a resource with ID 0 to select an entity whose AudioListener
/// component will be used.
pub struct SelectedListener(pub Entity);

// Wraps a source and signals to another thread when that source has ended.
struct EndSignalSource<I: Source>
where
    <I as Iterator>::Item: Sample,
{
    input: I,
    signal: Arc<AtomicBool>,
}

impl<I: Source> EndSignalSource<I>
where
    <I as Iterator>::Item: Sample,
{
    pub fn new(input: I, signal: Arc<AtomicBool>) -> EndSignalSource<I> {
        EndSignalSource { input, signal }
    }
}

impl<I: Source> Iterator for EndSignalSource<I>
where
    <I as Iterator>::Item: Sample,
{
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.input.next();
        if next.is_none() {
            self.signal.store(true, Ordering::Relaxed);
        }
        next
    }
}

impl<I: Source> Source for EndSignalSource<I>
where
    <I as Iterator>::Item: Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.input.channels()
    }

    fn samples_rate(&self) -> u32 {
        self.input.samples_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }
}

impl<'a> System<'a> for AudioSystem {
    type SystemData = (Fetch<'a, SelectedListener>,
     ReadStorage<'a, TransformComponent>,
     ReadStorage<'a, AudioListener>,
     WriteStorage<'a, AudioEmitter>);

    fn run(&mut self, (select_listener, transform, listener, mut audio_emitter): Self::SystemData) {

        // Process emitters and listener.
        if let Some(listener) = listener.get(select_listener.0) {
            if let Some(listener_transform) = transform.get(select_listener.0) {
                let listener_transform = Matrix4::from(listener_transform.0);
                let left_ear_position = listener_transform
                    .transform_point(Point3::from(listener.left_ear))
                    .into();
                let right_ear_position = listener_transform
                    .transform_point(Point3::from(listener.right_ear))
                    .into();
                for (transform, mut audio_emitter) in (&transform, &mut audio_emitter).join() {
                    let x = transform.0[3][0];
                    let y = transform.0[3][1];
                    let z = transform.0[3][2];
                    let emitter_position = [x, y, z];
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
                        let sink = SpatialSink::new(
                            &listener.output.endpoint,
                            emitter_position,
                            left_ear_position,
                            right_ear_position,
                        );
                        let atomic_bool = Arc::new(AtomicBool::new(false));
                        sink.append(EndSignalSource::new(source, atomic_bool.clone()));
                        audio_emitter.sinks.push((sink, atomic_bool));
                    }
                }
            }
        }
    }
}
