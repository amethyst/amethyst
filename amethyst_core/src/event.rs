use crate::ecs::*;

/// Read events generically
pub trait EventReader {
    /// The event type produced by the reader
    type Event: Clone + Send + Sync + 'static;

    /// Read events and append to the given Vec
    fn read(&mut self, _: &mut Resources, _: &mut Vec<Self::Event>);

    /// Setup event reader
    fn setup(&mut self, _res: &mut Resources) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shrev::{EventChannel, ReaderId};

    #[derive(Clone)]
    pub struct TestEvent;

    pub struct TestEventReader {
        reader: ReaderId<TestEvent>,
    }

    impl EventReader for TestEventReader {
        type Event = TestEvent;

        fn read(&mut self, resources: &mut Resources, data: &mut Vec<Self::Event>) {
            data.extend(
                resources
                    .get::<EventChannel<Self::Event>>()
                    .unwrap()
                    .read(&mut self.reader)
                    .cloned(),
            );
        }
    }

    #[derive(Clone)]
    pub struct OtherEvent;

    pub struct OtherEventReader {
        reader: ReaderId<OtherEvent>,
    }

    impl EventReader for OtherEventReader {
        type Event = OtherEvent;

        fn read(&mut self, resources: &mut Resources, data: &mut Vec<Self::Event>) {
            data.extend(
                resources
                    .get::<EventChannel<Self::Event>>()
                    .unwrap()
                    .read(&mut self.reader)
                    .cloned(),
            );
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

    impl EventReader for AggregateEventReader {
        type Event = AggregateEvent;

        fn read(&mut self, resources: &mut Resources, data: &mut Vec<Self::Event>) {
            data.extend(
                resources
                    .get::<EventChannel<TestEvent>>()
                    .unwrap()
                    .read(&mut self.test)
                    .cloned()
                    .map(Into::into),
            );
            data.extend(
                resources
                    .get::<EventChannel<OtherEvent>>()
                    .unwrap()
                    .read(&mut self.other)
                    .cloned()
                    .map(Into::into),
            );
        }
    }
}
