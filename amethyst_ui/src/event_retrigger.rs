use std::default::Default;

use amethyst_core::{
    shrev::{Event, EventChannel, ReaderId},
    specs::prelude::{Component, Read, ReadStorage, Resources, System, SystemData, Write},
};

use derivative::Derivative;

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
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct EventRetriggerSystem<T: EventRetrigger> {
    event_reader: Option<ReaderId<T::In>>,
}

impl<T> EventRetriggerSystem<T>
where
    T: EventRetrigger,
{
    /// Constructs a default `EventRetriggerSystem`. Since the `event_reader`
    /// will automatically be fetched when the system is set up, this should
    /// always be used to construct `EventRetriggerSystem`s.
    pub fn new() -> Self {
        Default::default()
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

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.event_reader = Some(res.fetch_mut::<EventChannel<T::In>>().register_reader());
    }

    fn run(&mut self, (in_channel, mut out_channel, retrigger): Self::SystemData) {
        let event_reader = self.event_reader.as_mut().expect(
            "`EventRetriggerSystem::setup` was not called before `EventRetriggerSystem::run`",
        );

        for event in in_channel.read(event_reader) {
            if let Some(entity_retrigger) = retrigger.get(event.get_target()) {
                entity_retrigger.apply(&event, &mut *out_channel);
            }
        }
    }
}
