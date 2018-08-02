# Event Channel

This chapter will be easier than the previous ones. 

While it is not essential to understand it to use amethyst, it can make your life much much easier in a lot of situations where using only data would make your code too complex.

## What is an event channel?

An `EventChannel` acts like a queue for any type that is `Send + Sync + 'static`.

It is a single producer/multiple receiver queue. This means that it works best when used with only a single "thing" (usually a system) producing events.
In most cases, the `EventChannel` should be stored in a global resource for ease of access. More on this later.

## Creating an event channel

Super simple!

```rust,ignore
    // In the following examples, we are going to use `MyEvent` values as events.
    #[derive(Debug)]
    pub enum MyEvent{
        A,
        B,
    }
    
    let mut channel = EventChannel::<MyEvent>::new();
```

## Writing events to the event channel

Single: 
```rust,ignore
    channel.single_write(MyEvent::A);
```

Multiple: 
```rust,ignore
    channel.iter_write(vec![MyEvent::A, MyEvent::A, MyEvent::B].iter());
```

## Reading events

This is the part where it becomes tricky.
To be able to track where each of the receiver's reading is at, the `EventChannel` needs to be aware of their presence.
This is done by registering a `ReaderId`.

```rust,ignore
    let mut reader = channel.register_reader();
```

Then, when you want to read the events:

```rust,ignore
    for event in channel.read_event(&mut reader){
        // The type of the event is inferred from the generic type
        // we assigned to the `EventChannel<MyEvent>` earlier when creating it.
        println!("Received event value of: {:?}", event);
    }
```
Note that you only need to have a read access to the channel when reading events.
It is the `ReaderId` that needs to be mutable to keep track of where your last read was.

## Patterns

When using the event channel, we usually re-use the same pattern over and over again to maximize parallelism.
It goes as follow:

Create the event channel and add it to to the world during `State` creation:
```rust,ignore
world.add_resource(
    EventChannel::<MyEvent>::new(),
);
```
_Note: You can also derive `Default`, this way you don't have to manually create your resource and add it. Resources implementing `Default` are automatically added to `Resources` when a `System` uses them (`Read` or `Write` in `SystemData`)._

In the **producer** `System`, get a mutable reference to your resource:
```rust,ignore
type SystemData = Write<'a, EventChannel<MyEvent>>;
```

In the **receiver** `System`s, you need to store the `ReaderId` somewhere.
```rust,ignore
struct ReceiverSystem {
    // The type inside of ReaderId should be the type of the event you are using.
    reader: Option<ReaderId<MyEvent>>,
}
```
and you also need to get read access:
```rust,ignore
    type SystemData = (Read<'a, EventChannel<MyEvent>>,);
```

Then, in the `System`'s setup method:
```rust,ignore
    // IMPORTANT: You need to setup your system data BEFORE you try to fetch the resource. Especially if you plan use `Default` to create your resource.
    Self::SystemData::setup(&mut res);
    self.reader = Some(res.fetch_mut::<EventChannel<MyEvent>>().unwrap().register_reader());
```

Finally, you can read events from your `System`.
```rust,ignore
    fn run (&mut self, my_event_channel: Self::SystemData) {
        for event in &my_event_channel.read(&mut self.reader) {
            info!("Received an event: {:?}", event);
        }
    }
```
