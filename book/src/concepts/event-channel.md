# Event Channel

This chapter will be easier than the previous ones. 

While it is not essential to understand it to use amethyst, it can make your life much much easier in a lot of situations where using only data would make your code too complex.

## What is an event channel?

An `EventChannel` acts like a queue for any type that is `Send + Sync + 'static`.

It is a single producer/multiple receiver queue. This means that it works best when used with only a single "thing" (usually a system) producing events.
In most cases, the `EventChannel` should be stored in a global resource for ease of access. More on this later.

## Creating an event channel

Super simple!

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::shrev::EventChannel;
    // In the following examples, we are going to use `MyEvent` values as events.
    #[derive(Debug)]
    pub enum MyEvent {
        A,
        B,
    }
    
    let mut channel = EventChannel::<MyEvent>::new();
```

## Writing events to the event channel

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

## Reading events

This is the part where it becomes tricky.
To be able to track where each of the receiver's reading is at, the `EventChannel` needs to be aware of their presence.
This is done by registering a `ReaderId`.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
# fn main() {
#   let mut channel = amethyst::shrev::EventChannel::<MyEvent>::new();
    let mut reader = channel.register_reader();
# }
```

Then, when you want to read the events:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
# fn main() {
#   let mut channel = amethyst::shrev::EventChannel::<MyEvent>::new();
#   let mut reader = channel.register_reader();
    for event in channel.read(&mut reader) {
        // The type of the event is inferred from the generic type
        // we assigned to the `EventChannel<MyEvent>` earlier when creating it.
        println!("Received event value of: {:?}", event);
    }
# }
```
Note that you only need to have a read access to the channel when reading events.
It is the `ReaderId` that needs to be mutable to keep track of where your last read was.

**IMPORTANT: The event channel automatically grows as events are added to it and only decreases in size once all readers have read through the older events.
This mean that if you create a `ReaderId` but don't read from it on each frame, the event channel will start to consume more and more memory.**

## Patterns

When using the event channel, we usually re-use the same pattern over and over again to maximize parallelism.
It goes as follow:

Create the event channel and add it to to the world during `State` creation:
```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::shrev::EventChannel;
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
# fn main() {
#   let mut world = amethyst::ecs::World::new();
world.add_resource(
    EventChannel::<MyEvent>::new(),
);
# }
```
_Note: You can also derive `Default`, this way you don't have to manually create your resource and add it. Resources implementing `Default` are automatically added to `Resources` when a `System` uses them (`Read` or `Write` in `SystemData`)._

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

Then, in the `System`'s setup method:
```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::shrev::{EventChannel, ReaderId};
# use amethyst::ecs::SystemData;
# #[derive(Debug)]
# pub enum MyEvent {
#   A,
#   B,
# }
# struct MySystem { reader: Option<ReaderId<MyEvent>>, }
# impl<'a> amethyst::ecs::System<'a> for MySystem {
#   type SystemData = ();
#   fn run(&mut self, _: Self::SystemData) { }
#   fn setup(&mut self, res: &mut amethyst::ecs::Resources) {
    // IMPORTANT: You need to setup your system data BEFORE you try to fetch the resource. Especially if you plan use `Default` to create your resource.
    Self::SystemData::setup(res);
    self.reader = Some(res.fetch_mut::<EventChannel<MyEvent>>().register_reader());
#   }
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
#   reader: Option<amethyst::shrev::ReaderId<MyEvent>>,
# }
# impl<'a> amethyst::ecs::System<'a> for MySystem {
#   type SystemData = Read<'a, EventChannel<MyEvent>>;
    fn run(&mut self, my_event_channel: Self::SystemData) {
        for event in my_event_channel.read(self.reader.as_mut().unwrap()) {
            println!("Received an event: {:?}", event);
        }
    }
# }
```
