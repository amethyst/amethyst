# Watching Input Events From Systems

You can directly listen to input [event channels](../concepts/event-channel.md). This can be useful if you need to detect an key press or action as it happens.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::{
#     ecs::{System, SystemData, Read},
#     prelude::*,
# };
use amethyst::{
    input::{InputEvent, StringBindings},
    shrev::{EventChannel, ReaderId},
};

pub struct CustomInputSystem {
    input_event_rid: ReaderId<InputEvent<StringBindings>>,
}

impl CustomInputSystem {
    pub fn new(world: &mut World) -> Self {
        <Self as System<'_>>::SystemData::setup(world);
        let input_event_rid = world.fetch_mut::<EventChannel<InputEvent<StringBindings>>>().register_reader();
        Self { input_event_rid }
    }
}

impl<'a> System<'a> for CustomInputSystem {
    type SystemData = Read<'a, EventChannel<InputEvent<StringBindings>>>;
    fn run(&mut self, event_channel: Self::SystemData) {
        for event in event_channel.read(&mut self.input_event_rid) {
            if let InputEvent::KeyPressed { key_code, scancode } = event {
                println!("{:?} was pressed", key_code);
            }
        }
    }
}
```

Setting up the system is a bit different as you'll need to store a reader id on the system and implement a new function (see the [event channel](../concepts/event-channel.md) guide for more details and another example of doing this) on the system to initialize that reader id. We declare our reader id (`input_event_rid`) with the events we want to listen to, in this case were listening to input events, so we're going to use `InputEvent` and use the `StringBindings`, but you would use whatever binding struct you're using in your game here.

Now we can create a `Read` in our system data for the input event channel and in our systems run function we can iterate over all the input events the occured. Take a look at the [`InputEvent`][doc_input_event] enum for all the available input events.

There we have it, now you can read input events as they occur, be sure to look at the [`event channel`](../concepts/event-channel.md) section for a better guide on using event channels and enjoy listening to input events!

[doc_input_event]: https://docs.amethyst.rs/stable/amethyst_input/enum.InputEvent.html
