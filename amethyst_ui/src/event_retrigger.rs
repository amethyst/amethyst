use amethyst_core::{
    ecs::*,
    shrev::{Event, EventChannel, ReaderId},
};
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::event::TargetedEvent;
use amethyst_core::ecs::systems::ParallelRunnable;

/// Describes anything that can receive events one by one or in batches. This
/// lets whoever wants to receive triggered events decide on how they
/// want to receive them, instead of forcing them to construct certain
/// data structures such as a `Vec`.
pub trait EventReceiver<T> {
    /// Receive a single event
    fn receive_one(&mut self, value: &T);
    /// Receive an iterator over several events
    fn receive(&mut self, values: &[T]);
}

impl<T> EventReceiver<T> for EventChannel<T>
where
    T: Clone + Event,
{
    fn receive_one(&mut self, value: &T) {
        self.single_write(value.clone());
    }

    fn receive(&mut self, values: &[T]) {
        self.iter_write(values.iter().cloned());
    }
}

/// Trait that denotes which event gets retriggered to which other event and how
pub trait EventRetrigger {
    /// Event type that causes retrigger
    type In: Clone + Send + Sync + TargetedEvent;
    /// Event type that gets retriggered
    type Out: Clone + Send + Sync;

    /// Denotes how In events retriggers Out events
    fn apply<R>(&self, event: &Self::In, out: &mut R)
    where
        R: EventReceiver<Self::Out>;
}

/// Links up the given in- and output types' `EventChannel`s listening
/// to incoming events and calling `apply` on the respective `Retrigger`
/// components.
#[derive(Debug)]
pub struct EventRetriggerSystem<T: EventRetrigger> {
    event_reader: ReaderId<T::In>,
}

impl<T> EventRetriggerSystem<T>
where
    T: EventRetrigger,
{
    /// Constructs a default `EventRetriggerSystem`. Since the `event_reader`
    /// will automatically be fetched when the system is set up, this should
    /// always be used to construct `EventRetriggerSystem`s.
    pub fn new(event_reader: ReaderId<T::In>) -> Self {
        Self { event_reader }
    }

    pub fn build(&mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("EventRetriggerSystem")
                .read_resource::<EventChannel<T::In>>()
                .write_resource::<EventChannel<EventChannel<T::Out>>>()
                .read_resource::<T>()
                .build( move | _commands, _world, (in_channel, mut out_channel, retrigger), _| {
                    #[cfg(feature = "profiler")]
                    profile_scope!("event_retrigger_system");
                    let event_reader = &mut self.event_reader;
                    for event in in_channel.read(event_reader) {
                        if let Some(entity_retrigger) = retrigger.get(event.get_target()) {
                            entity_retrigger.apply(&event, &mut *out_channel);
                        }
                    }
                }
            )
        )
    }
}
