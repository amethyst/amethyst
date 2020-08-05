# Event Channel

## What is an event channel?

An `EventChannel` is a broadcast queue of events. Events may be any type that implements `Send + Sync + 'static`.

Typically, `EventChannel`s are inserted as resources in the `World`.

## Examples

### Creating an event channel

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::shrev::EventChannel;
// In the following examples, `MyEvent` is the event type of the channel.
#[derive(Debug)]
pub enum MyEvent {
    A,
    B,
}

let mut channel = EventChannel::<MyEvent>::new();
```

### Writing events to the event channel

Single:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
# fn main() {
#   let mut channel = amethyst::shrev::EventChannel::<MyEvent>::new();
    channel.single_write(MyEvent::A);
# }
```

Multiple:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
# fn main() {
#   let mut channel = amethyst::shrev::EventChannel::<MyEvent>::new();
    channel.iter_write(vec![MyEvent::A, MyEvent::A, MyEvent::B].into_iter());
# }
```

### Reading events

`EventChannel`s guarantee sending events in order to each reader.

To subscribe to events, register a reader against the `EventChannel` to receive a `ReaderId`:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
# fn main() {
#   let mut channel = amethyst::shrev::EventChannel::<MyEvent>::new();
let mut reader_id = channel.register_reader();
# }
```

When reading events, pass the `ReaderId` in:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
# fn main() {
#   let mut channel = amethyst::shrev::EventChannel::<MyEvent>::new();
#   let mut reader_id = channel.register_reader();
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

When using the event channel, we usually re-use the same pattern over and over again to maximize parallelism.
It goes as follow:

In the **producer** `System`, get a mutable reference to your resource:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::Write;
# use amethyst::shrev::EventChannel;
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
# struct MySystem;
# impl<'a> amethyst::ecs::System<'a> for MySystem {
type SystemData = Write<'a, EventChannel<MyEvent>>;
#   fn run(&mut self, _: Self::SystemData) { }
# }
```

In the **receiver** `System`s, you need to store the `ReaderId` somewhere.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::shrev::ReaderId;
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
struct ReceiverSystem {
    // The type inside of ReaderId should be the type of the event you are using.
    reader: Option<ReaderId<MyEvent>>,
}
```

and you also need to get read access:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::Read;
# use amethyst::shrev::EventChannel;
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
# struct MySystem;
# impl<'a> amethyst::ecs::System<'a> for MySystem {
    type SystemData = Read<'a, EventChannel<MyEvent>>;
#   fn run(&mut self, _: Self::SystemData) { }
# }
```

Then, in the `System`'s `new` method:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::shrev::{EventChannel, ReaderId};
# use amethyst::ecs::{System, SystemData, World};
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
# struct MySystem { reader_id: ReaderId<MyEvent>, }
#
impl MySystem {
    pub fn new(world: &mut World) -> Self {
        <Self as System<'_>>::SystemData::setup(world);
        let reader_id = world.fetch_mut::<EventChannel<MyEvent>>().register_reader();
        Self { reader_id }
    }
}
#
# impl<'a> amethyst::ecs::System<'a> for MySystem {
#   type SystemData = ();
#   fn run(&mut self, _: Self::SystemData) { }
# }
```

Finally, you can read events from your `System`.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::Read;
# use amethyst::shrev::EventChannel;
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
# struct MySystem {
#   reader_id: amethyst::shrev::ReaderId<MyEvent>,
# }
impl<'a> amethyst::ecs::System<'a> for MySystem {
    type SystemData = Read<'a, EventChannel<MyEvent>>;
    fn run(&mut self, my_event_channel: Self::SystemData) {
        for event in my_event_channel.read(&mut self.reader_id) {
            println!("Received an event: {:?}", event);
        }
    }
}
```
