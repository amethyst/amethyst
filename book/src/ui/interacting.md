# Interaction!

Now that we have a button we can interact with, let's see how we actually do this.
We will show you how to do this in two ways. One way will be interaction through a system,
and the latter interaction through `handle_event` method of your active state.

## Creating the system

Let's start of with some boilerplate code: 

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::System;

pub struct SimpleButtonSystem;

impl<'s> System<'s> for SimpleButtonSystem {
    type SystemData = ();

    fn run(&mut self, data: Self::SystemData) {
	
    }
}
```

This was shown in previous [chapters](../concepts/system/system_initialization.html).
The way you will be able to read the generated 
events is with a [ReaderId](https://docs.amethyst.rs/master/specs/prelude/struct.ReaderId.html).
The `ReaderId` is added as a field to the system struct.

The events we want to read are of type [UiEvent](https://docs.amethyst.rs/master/amethyst_ui/struct.UiEvent.html).
We also need to fetch the [EventChannel](https://docs.amethyst.rs/master/shrev/struct.EventChannel.html) in our `SystemData`, 
since the `ReaderId` actually pulls (reads) information  from the `EventChannel`.

Adding it up, it should look like this: 

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::{System, Read};
# use amethyst::shrev::{EventChannel, ReaderId};
# use amethyst::ui::UiEvent;

pub struct SimpleButtonSystem {
    reader_id: ReaderId<UiEvent>,
}

impl<'s> System<'s> for SimpleButtonSystem {
    type SystemData = Read<'s, EventChannel<UiEvent>>;

    fn run(&mut self, events: Self::SystemData) {

    }
}
```

We also need a constructor for our system:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::{System, Read};
# use amethyst::shrev::{EventChannel, ReaderId};
# use amethyst::ui::UiEvent;
#
# pub struct SimpleButtonSystem {
#    reader_id: ReaderId<UiEvent>,
# }
#
# impl<'s> System<'s> for SimpleButtonSystem {
#     type SystemData = Read<'s, EventChannel<UiEvent>>;
#
#     fn run(&mut self, events: Self::SystemData) {
#
#     }
# }
impl SimpleButtonSystem {
    pub fn new(reader_id: ReaderId<UiEvent>) -> Self {
        Self {
            reader_id,	
        }
    }
}
```

To add the system to our game data we actually need a `SystemDesc` implementation for our system:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::{System, World, Read, Write, SystemData};
# use amethyst::core::SystemDesc;
# use amethyst::shrev::{EventChannel, ReaderId};
# use amethyst::ui::UiEvent;
# pub struct SimpleButtonSystem {
#    reader_id: ReaderId<UiEvent>,
# }
#
# impl<'s> System<'s> for SimpleButtonSystem {
#     type SystemData = Read<'s, EventChannel<UiEvent>>;
#
#     fn run(&mut self, events: Self::SystemData) {
#
#     }
# }
# impl SimpleButtonSystem {
#     pub fn new(reader_id: ReaderId<UiEvent>) -> Self {
#        Self {
#             reader_id,	
#       }
#     }
# }
pub struct SimpleButtonSystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, SimpleButtonSystem> for SimpleButtonSystemDesc {
    fn build(self, world: &mut World) -> SimpleButtonSystem {
        let mut event_channel = <Write<EventChannel<UiEvent>>>::fetch(world);
        let reader_id = event_channel.register_reader();

        SimpleButtonSystem::new(reader_id)
    }
}
```
Now that this is done we can start reading our events!

In our systems `run` method we are going to loop through all the events:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::{System, World, Read};
# use amethyst::core::SystemDesc;
# use amethyst::shrev::{EventChannel, ReaderId};
# use amethyst::ui::UiEvent;
# pub struct SimpleButtonSystem {
#    reader_id: ReaderId<UiEvent>,
# }
#
# impl<'s> System<'s> for SimpleButtonSystem {
#     type SystemData = Read<'s, EventChannel<UiEvent>>;
#
fn run(&mut self, events: Self::SystemData) {
    for event in events.read(&mut self.reader_id) {
        println!("{:?}", event);	
    }
}
# }
```

Let's try and change the text color when the button receives a hovered event!

Firstly we need to fetch two more components that 
we used for our entity - `UiTransform` and `UiText`.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::{System, World, Read, ReadStorage, WriteStorage};
# use amethyst::core::SystemDesc;
# use amethyst::shrev::{EventChannel, ReaderId};
# use amethyst::ui::{UiTransform, UiText, UiEvent};
# pub struct SimpleButtonSystem {
#    reader_id: ReaderId<UiEvent>,
# }
#
# impl<'s> System<'s> for SimpleButtonSystem {
type SystemData = Read<'s, EventChannel<UiEvent>>;
#
# fn run(&mut self, events: Self::SystemData) {
#     for event in events.read(&mut self.reader_id) {
#         println!("{:?}", event);	
#     }
# }
# }
```

Usage of `WriteStorage<'s, UiText>` is needed since we will be changing 
the color that is the property of the `UiText` component.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::{System, World, Read, ReadStorage, WriteStorage};
# use amethyst::core::SystemDesc;
# use amethyst::shrev::{EventChannel, ReaderId};
# use amethyst::ui::{UiTransform, UiText, UiEvent, UiEventType};
# pub struct SimpleButtonSystem {
#    reader_id: ReaderId<UiEvent>,
# }
#
# impl<'s> System<'s> for SimpleButtonSystem {
# type SystemData = (
#     Read<'s, EventChannel<UiEvent>>,
#     ReadStorage<'s, UiTransform>,
#     WriteStorage<'s, UiText>,
# );
#
fn run(&mut self, (events, transforms, mut texts): Self::SystemData) {
    for event in events.read(&mut self.reader_id) {
        let button_text = texts.get_mut(event.target).unwrap();

        match event.event_type {
            UiEventType::HoverStart => { 
                button_text.color = [1.0, 1.0, 1.0, 1.0]; 
            },
            UiEventType::HoverStop  => { 
                button_text.color = [1.0, 1.0, 1.0, 0.5]; 
            },
            _ => {},
        }   
    }
}
# }
```

The `HoverStart` and `HoverStop` are emitted once, upon the cursor 
entering the transform area and exiting respectively. 

This will brighten the button when hovering over it, and dim it otherwise.

**Please note** that you would likely have some kind of checks in order to know 
for which button the event is generated. 
We haven't performed any here since we only have one button, so all generated 
events are tied to that button.

 
Basically you want all the magic happening in the systems, like fading
effects, scaling effect and such. 

In theory you could set up a connection between the system and the state
like a resource, which will determine the change of the state.
Eventhough possible, it is not recommended. That's why now 
we will go through managing input through the state.





















