# Event Channel

## What is an event channel?

An `EventChannel` is a broadcast queue of events. Events may be any type that implements `Send + Sync + 'static`.

Typically, `EventChannel`s are inserted as resources in `Resources`.

## Examples

### Creating an event channel

```rust
use amethyst::shrev::EventChannel;

// In the following examples, `MyEvent` is the event type.
#[derive(Debug)]
pub enum MyEvent {
    A,
    B,
}

# fn main() {
    let mut channel = EventChannel::<MyEvent>::new();
# }
```

### Writing events to the event channel

Single:

```rust
# use amethyst::shrev::EventChannel;
# 
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
# 
# fn main() {
#   let mut channel = EventChannel::<MyEvent>::new();
    channel.single_write(MyEvent::A);
# }
```

Multiple:

```rust
# use amethyst::shrev::EventChannel;
# 
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
# 
# fn main() {
#   let mut channel = EventChannel::<MyEvent>::new();
    channel.iter_write(vec![MyEvent::A, MyEvent::A, MyEvent::B].into_iter());
# }
```

### Reading events

`EventChannel`s guarantee sending events in order to each reader.

To subscribe to events, register a reader against the `EventChannel` to receive a `ReaderId`:

```rust
# use amethyst::shrev::EventChannel;
# 
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
# 
# fn main() {
#   let mut channel = EventChannel::<MyEvent>::new();
    let mut reader_id = channel.register_reader();
    for event in channel.read(&mut reader_id) {
        // The type of the event is inferred from the generic type
        // we assigned to the `EventChannel<MyEvent>` earlier when creating it.
        println!("Received event value of: {:?}", event);
    }
# }
```

Note that you only need to have a read access to the channel when reading events.
It is the `ReaderId` that needs to be mutable to keep track of where your last read was.

> **IMPORTANT:** The event channel automatically grows as events are added to it and only decreases in size once all readers have read through the older events.
>
> This mean that if you create a `ReaderId` but don't read from it on each frame, the event channel will start to consume more and more memory.

## Patterns

When using the event channel, we re-use the same pattern over and over again to maximize parallelism.
It goes as follow:

In the **producer** `System`, get a mutable reference to your resource:

```rust
# use amethyst::shrev::EventChannel;
# 
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
# 
struct MyEventProducerSystem;

use amethyst::ecs::{ParallelRunnable, System, SystemBuilder};

impl System for MyEventProducerSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("MyEventProducerSystem")
                .write_resource::<EventChannel<MyEvent>>()
                .build(move |_, _, my_event_channel, _| {
                    my_event_channel.single_write(MyEvent::A);
                }),
        )
    }
}
```

In the **consumer** `System`s, you need to store the `ReaderId`.

```rust
# use amethyst::{
#   ecs::{System, World},
#   shrev::{EventChannel, ReaderId},
# };
# 
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
# 
struct MyEventConsumerSystem {
    reader_id: ReaderId<MyEvent>,
}

impl MyEventConsumerSystem {
    pub fn new(resources: &mut Resources) -> Self {
        let reader_id = resources
            .get_mut::<EventChannel<MyEvent>>()
            .register_reader();
        Self { reader_id }
    }
}

fn build(mut self) -> Box<dyn ParallelRunnable> {
    Box::new(
        SystemBuilder::new("MyEventConsumerSystem")
            .read_resource::<EventChannel<MyEvent>>()
            .build(move |_, _, my_event_channel, _| {
                for event in my_event_channel.read(&mut self.reader) {
                    println!("Received an event: {:?}", event);
                }
            }),
    )
}
```
