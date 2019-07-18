use amethyst_core::{
    ecs::prelude::{Component, Read, ReadStorage, System, SystemData, World, Write},
    shrev::{Event, EventChannel, ReaderId},
};

use crate::event::TargetedEvent;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

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

pub trait EventRetrigger: Component {
    type In: Clone + Send + Sync + TargetedEvent;
    type Out: Clone + Send + Sync;

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
    pub fn new(mut world: &mut World) -> Self {
        <Self as System<'_>>::SystemData::setup(&mut world);
        let event_reader = world.fetch_mut::<EventChannel<T::In>>().register_reader();
        Self { event_reader }
    }
}

impl<'s, T> System<'s> for EventRetriggerSystem<T>
where
    T: EventRetrigger,
{
    type SystemData = (
        Read<'s, EventChannel<T::In>>,
        Write<'s, EventChannel<T::Out>>,
        ReadStorage<'s, T>,
    );

    fn run(&mut self, (in_channel, mut out_channel, retrigger): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("event_retrigger_system");

        let event_reader = &mut self.event_reader;

        for event in in_channel.read(event_reader) {
            if let Some(entity_retrigger) = retrigger.get(event.get_target()) {
                entity_retrigger.apply(&event, &mut *out_channel);
            }
        }
    }
}
