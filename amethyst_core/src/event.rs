use crate::ecs::{SystemData, World};

/// Read events generically
pub trait EventReader<'a> {
    /// SystemData needed to read the event(s)
    type SystemData: SystemData<'a>;
    /// The event type produced by the reader
    type Event: Clone + Send + Sync + 'static;

    /// Read events from the linked `SystemData` and append to the given Vec
    fn read(&mut self, _: Self::SystemData, _: &mut Vec<Self::Event>);

    /// Read events from `World` and append to the given `Vec`
    fn read_from_world(&mut self, world: &'a World, events: &mut Vec<Self::Event>) {
        self.read(world.system_data(), events);
    }

    /// Setup event reader
    fn setup(&mut self, res: &mut World) {
        Self::SystemData::setup(res);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ecs::Read,
        shrev::{EventChannel, ReaderId},
    };

    use super::*;

    #[derive(Clone)]
    pub struct TestEvent;

    pub struct TestEventReader {
        reader: ReaderId<TestEvent>,
    }

    impl<'a> EventReader<'a> for TestEventReader {
        type SystemData = Read<'a, EventChannel<TestEvent>>;
        type Event = TestEvent;

        fn read(&mut self, events: Self::SystemData, data: &mut Vec<TestEvent>) {
            data.extend(events.read(&mut self.reader).cloned());
        }
    }

    #[derive(Clone)]
    pub struct OtherEvent;

    pub struct OtherEventReader {
        reader: ReaderId<OtherEvent>,
    }

    impl<'a> EventReader<'a> for OtherEventReader {
        type SystemData = Read<'a, EventChannel<OtherEvent>>;
        type Event = OtherEvent;

        fn read(&mut self, events: Self::SystemData, data: &mut Vec<OtherEvent>) {
            data.extend(events.read(&mut self.reader).cloned());
        }
    }

    #[derive(Clone)]
    pub enum AggregateEvent {
        Test(TestEvent),
        Other(OtherEvent),
    }

    impl From<TestEvent> for AggregateEvent {
        fn from(event: TestEvent) -> Self {
            AggregateEvent::Test(event)
        }
    }

    impl From<OtherEvent> for AggregateEvent {
        fn from(event: OtherEvent) -> Self {
            AggregateEvent::Other(event)
        }
    }

    #[allow(unused)]
    pub struct AggregateEventReader {
        test: ReaderId<TestEvent>,
        other: ReaderId<OtherEvent>,
    }

    impl<'a> EventReader<'a> for AggregateEventReader {
        type SystemData = (
            <TestEventReader as EventReader<'a>>::SystemData,
            <OtherEventReader as EventReader<'a>>::SystemData,
        );
        type Event = AggregateEvent;

        fn read(&mut self, system_data: Self::SystemData, data: &mut Vec<AggregateEvent>) {
            data.extend(system_data.0.read(&mut self.test).cloned().map(Into::into));
            data.extend(system_data.1.read(&mut self.other).cloned().map(Into::into));
        }
    }
}
