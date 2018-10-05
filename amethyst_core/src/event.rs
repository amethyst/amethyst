use specs::{World, SystemData};

/// Read events generically
pub trait EventReader<'a> {
    type SystemData: SystemData<'a>;
    type Event: Clone + Send + Sync + 'static;

    fn read(&mut self, _: Self::SystemData, _: &mut Vec<Self::Event>);

    fn read_from_world(&mut self, world: &'a World, events: &mut Vec<Self::Event>) {
        self.read(world.system_data(), events);
    }

    fn build(world: &mut World) -> Self;
}

#[cfg(test)]
mod tests {
    use shrev::{EventChannel, ReaderId};
    use specs::Read;

    use super::*;

    #[derive(Clone)]
    pub struct TestEvent;

    /// TODO: Create macro for this
    pub struct TestEventReader {
        reader: ReaderId<TestEvent>
    }

    impl<'a> EventReader<'a> for TestEventReader {
        type SystemData = Read<'a, EventChannel<TestEvent>>;
        type Event = TestEvent;

        fn read(&mut self, events: Self::SystemData, data: &mut Vec<TestEvent>) {
            data.extend(events.read(&mut self.reader).cloned());
        }

        fn build(world: &mut World) -> Self {
            TestEventReader {
                reader: world.write_resource::<EventChannel<TestEvent>>().register_reader()
            }
        }
    }

    #[derive(Clone)]
    pub struct OtherEvent;

    pub struct OtherEventReader {
        reader: ReaderId<OtherEvent>
    }

    impl<'a> EventReader<'a> for OtherEventReader {
        type SystemData = Read<'a, EventChannel<OtherEvent>>;
        type Event = OtherEvent;

        fn read(&mut self, events: Self::SystemData, data: &mut Vec<OtherEvent>) {
            data.extend(events.read(&mut self.reader).cloned());
        }

        fn build(world: &mut World) -> Self {
            OtherEventReader {
                reader: world.write_resource::<EventChannel<OtherEvent>>().register_reader()
            }
        }
    }

    #[derive(Clone)]
    pub enum AggregateEvent {
        Test(TestEvent),
        Other(OtherEvent),
    }

    impl From<TestEvent> for AggregateEvent {
        fn from(event: TestEvent) -> Self {
            AggregateEvent::Test(event.clone())
        }
    }

    impl From<OtherEvent> for AggregateEvent {
        fn from(event: OtherEvent) -> Self {
            AggregateEvent::Other(event)
        }
    }

    pub struct AggregateEventReader {
        test: ReaderId<TestEvent>,
        other: ReaderId<OtherEvent>,
    }

    impl<'a> EventReader<'a> for AggregateEventReader {
        type SystemData = (<TestEventReader as EventReader<'a>>::SystemData, <OtherEventReader as EventReader<'a>>::SystemData);
        type Event = AggregateEvent;

        fn read(&mut self, system_data: Self::SystemData, data: &mut Vec<AggregateEvent>) {
            data.extend(system_data.0.read(&mut self.test).cloned().map(Into::into));
            data.extend(system_data.1.read(&mut self.other).cloned().map(Into::into));
        }

        fn build(world: &mut World) -> Self {
            AggregateEventReader {
                test: world.write_resource::<EventChannel<TestEvent>>().register_reader(),
                other: world.write_resource::<EventChannel<OtherEvent>>().register_reader(),
            }
        }
    }
}
