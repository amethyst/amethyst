use assets::AssetStorage;
use audio::{AudioSink, Source, SourceHandle};
use ecs::{Fetch, System};
use ecs::common::Errors;

/// Calls a closure if the `AudioSink` is empty.
pub struct DjSystem<F> {
    f: F,
}

impl<F> DjSystem<F> {
    /// Creates a new `DjSystem` with the music picker being `f`.
    pub fn new(f: F) -> Self {
        DjSystem { f }
    }
}

impl<'a, F> System<'a> for DjSystem<F>
where
    F: FnMut() -> Option<SourceHandle>,
{
    type SystemData = (
        Fetch<'a, AssetStorage<Source>>,
        Fetch<'a, Errors>,
        Fetch<'a, AudioSink>,
    );

    fn run(&mut self, (storage, errors, sink): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("dj_system");
        if sink.empty() {
            if let Some(source) = (&mut self.f)().and_then(|h| storage.get(&h)) {
                errors.execute(|| sink.append(source));
            }
        }
    }
}
