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

Setting up the system has extra requirements. You will need to store a `ReaderId` in your system struct used to read the type of input events you care about. In this example, our struct field is called `input_event_rid` and the type of event we are listening to is `InputEvent<StringBindings>`, but you would use whatever event type you would want to use in your own game.  You need to initialize your `ReaderID` in the system's `new` method (see the [event channel](../concepts/event-channel.md) guide for more details and another example of doing this).

Next let's take a look at our implementation of the `System` trait. We set the `SystemData` associated type to a `Read` storage for the type of event channel we want.  In the `run` method we iterate over all the input events the occured. Take a look at the [`InputEvent`][doc_input_event] enum for all the available input events you can handle.

There you have it! Now you can read input events as they occur. Be sure to look at the [`event channel`](../concepts/event-channel.md) section for a better guide on using event channels and enjoy listening to input events!

[doc_input_event]: https://docs.amethyst.rs/stable/amethyst_input/enum.InputEvent.html
