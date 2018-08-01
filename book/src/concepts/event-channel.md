# Event Channel

This chapter will be easier than the previous ones. 

While it is not essential to understand it to use amethyst, it can make your life much much easier in a lot of situations where using only data would make your code too complex.

## What is an event channel?

An `EventChannel` acts like a queue for any type that implements `Event`.
The `Event` type is automatically implemented for all types that are `Send`, `Sync` and have a static lifetime.

It is a single producer/multiple receiver. This means that it should be used when you only have a single "thing" (usually a system) producing events.
In most case, the `EventChannel` should be stored in a global resource for ease of access. More on this later.


The only exception to this is when you have a **SINGLE** external thread producing the events and you want your game to have a read access over the `EventChannel`.
In this specific case, you can create a resource holding a Arc<Mutex<EventChannel<T>>>.

If your use case involves multiple event producers, you should use some other type of event queuing method.
In those case, you can use a `System` reading from a mpsc channel and moving the events into a `EventChannel`.
You can also use other rust crates for that purpose.

## Creating an event channel

Super simple!

```rust,ignore
    // In the following examples, we are going to use i32 values as events.
    let mut channel = EventChannel::<i32>::new();
```

## Writing events to the event channel

Single: 
```rust,ignore
    channel.single_write(55);
```

Multiple: 
```rust,ignore
    channel.iter_write(vec![1,2,3,4,5].iter());
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
        // The type of the event is infered from the generic type
        // we assigned to the `EventChannel<i32>` earlier when creating it.
        println!("Received i32 event value of: {}", event);
    }
```
Note that you only need to have a read access to the channel when reading events.
It is the `ReaderId` that needs to be mutable to keep track of where your last read was.

## Patterns

When using the event channel, we usually re-use the same pattern over and over to maximize parallelism.
It goes as follow:

Create a resource holding the event channel:
```rust,ignore
struct MyEventChannel {
    // Remember, you can use almost any type instead of i32.
    channel: EventChannel<i32>,
}
```

Add the resource to the world during `State` creation:
```rust,ignore
world.add_resource(
    MyEventChannel {
        channel: EventChannel::<i32>::new(),
    }
);
```
_Note: You can also derive `Default`, this way you don't have to manually create your resource and add it._

In the producer `System`, get a mutable reference to your resource:
```rust,ignore
// Since we have a single element, we need the last "," to ensure `SystemData` is considered as a struct.
type SystemData = (Write<'a, MyEventChannel>,);
```

In the receiver `System`s, you need to store the `ReaderId` somewhere.
```rust,ignore
struct ReceiverSystem {
    reader: Option<ReaderId<i32>>,
}
```

Then, in the `System`'s setup method:
```rust,ignore
    // IMPORTANT: You need to setup your system data BEFORE you try to fetch the resource. Especially if you plan use `Default` to create your resource.
    Self::SystemData::setup(&mut res);
    self.reader = Some(res.fetch_mut::<MyEventChannel>().unwrap().register_reader());
```

Finally, you can read events from your `System`.
```rust,ignore
    fn run (&mut self, (my_event_channel,): Self::SystemData) {
        for event in &my_event_channel.read(&mut self.reader) {
            info!("Received an event: {}", event);
        }
    }
```
