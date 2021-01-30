use std::ops::DerefMut;

use amethyst_core::{
    ecs::{storage::Component, *},
    shrev::{Event, EventChannel, ReaderId},
};
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::event::TargetedEvent;

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
pub trait EventRetrigger: Component {
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
pub struct EventRetriggerSystem<T: EventRetrigger + 'static> {
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
}

impl<T: EventRetrigger + 'static> System for EventRetriggerSystem<T> {
    fn build(mut self) -> Box<dyn ParallelRunnable> {
        let system_name = format!("{}System", std::any::type_name::<T>());
        Box::new(
            SystemBuilder::new(system_name)
                .read_resource::<EventChannel<T::In>>()
                .write_resource::<EventChannel<T::Out>>()
                .with_query(<(Entity, &mut T)>::query())
                .build(
                    move |_commands, world, (in_channel, out_channel), retrigger| {
                        #[cfg(feature = "profiler")]
                        profile_scope!("event_retrigger_system");
                        let event_reader = &mut self.event_reader;
                        for event in in_channel.read(event_reader) {
                            if let Ok((_, entity_retrigger)) =
                                retrigger.get_mut(world, event.get_target())
                            {
                                entity_retrigger.apply(&event, out_channel.deref_mut());
                            }
                        }
                    },
                ),
        )
    }
}
