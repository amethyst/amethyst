# Interaction!

Now that we have a button we can interact with, let's see how we actually do this.
We will show you how to do this in two ways. One way will be interaction through a system,
and the latter interaction through `handle_event` method of your active state.

## Creating the system

Let's start of with some boilerplate code:

```rust
# use amethyst::ecs::System;

pub struct SimpleButtonSystem;

impl System for SimpleButtonSystem {
    fn run(&mut self, data: Self::SystemData) {}
}
```

This was shown in previous [chapters][sys_ini].
The way you will be able to read the generated
events is with a [ReaderId].
The `ReaderId` is added as a field to the system struct.

The events we want to read are of type [UiEvent].
We also need to fetch the [EventChannel] in our `SystemData`,
since the `ReaderId` actually pulls (reads) information  from the `EventChannel`.

Adding it up, it should look like this:

```rust
# use amethyst::ecs::{System};
# use amethyst::shrev::{EventChannel, ReaderId};
# use amethyst::ui::UiEvent;

pub struct SimpleButtonSystem {
    reader_id: ReaderId<UiEvent>,
}

impl System for SimpleButtonSystem {
    type SystemData = .read_resource::<EventChannel<UiEvent>>();

    fn run(&mut self, events: Self::SystemData) {}
}
```

We also need a constructor for our system:

```rust
# use amethyst::ecs::{System};
# use amethyst::shrev::{EventChannel, ReaderId};
# use amethyst::ui::UiEvent;
# 
# pub struct SimpleButtonSystem {
#   reader_id: ReaderId<UiEvent>,
# }
# 
# impl System for SimpleButtonSystem {
#   type SystemData = .read_resource::<EventChannel<UiEvent>>();
# 
#   fn run(&mut self, events: Self::SystemData) {}
# }
impl SimpleButtonSystem {
    pub fn new(reader_id: ReaderId<UiEvent>) -> Self {
        Self { reader_id }
    }
}
```

To add the system to our game data we actually need a `SystemDesc` implementation for our system:

```rust
# use amethyst::ecs::{System, World};
# use amethyst::shrev::{EventChannel, ReaderId};
# use amethyst::ui::UiEvent;
# pub struct SimpleButtonSystem {
#   reader_id: ReaderId<UiEvent>,
# }
# 
# impl System for SimpleButtonSystem {
#   type SystemData = .read_resource::<EventChannel<UiEvent>>();
# 
#   fn run(&mut self, events: Self::SystemData) {}
# }
# impl SimpleButtonSystem {
#   pub fn new(reader_id: ReaderId<UiEvent>) -> Self {
#       Self { reader_id }
#   }
# }
pub struct SimpleButtonSystemDesc;

impl SystemDesc<'a, 'b, SimpleButtonSystem> for SimpleButtonSystemDesc {
    fn build(self, world: &mut World) -> SimpleButtonSystem {
        let mut event_channel = <Write<EventChannel<UiEvent>>>::fetch(world);
        let reader_id = event_channel.register_reader();

        SimpleButtonSystem::new(reader_id)
    }
}
```

Now that this is done we can start reading our events!

In our systems `run` method we are going to loop through all the events:

```rust
# use amethyst::ecs::{System, World};
# use amethyst::shrev::{EventChannel, ReaderId};
# use amethyst::ui::UiEvent;
# pub struct SimpleButtonSystem {
#   reader_id: ReaderId<UiEvent>,
# }
# 
# impl System for SimpleButtonSystem {
#   type SystemData = .read_resource::<EventChannel<UiEvent>>();
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

```rust
# use amethyst::ecs::{System, World};
# use amethyst::shrev::{EventChannel, ReaderId};
# use amethyst::ui::{UiEvent, UiText, UiTransform};
# pub struct SimpleButtonSystem {
#   reader_id: ReaderId<UiEvent>,
# }
# 
# impl System for SimpleButtonSystem {
    type SystemData = .read_resource::<EventChannel<UiEvent>>();
#   fn run(&mut self, events: Self::SystemData) {
#       for event in events.read(&mut self.reader_id) {
#           println!("{:?}", event);
#       }
#   }
# }
```

Usage of `.write_component::<UiText>` is needed since we will be changing
the color that is the property of the `UiText` component.

```rust
# use amethyst::ecs::{System, World};
# use amethyst::shrev::{EventChannel, ReaderId};
# use amethyst::ui::{UiEvent, UiEventType, UiText, UiTransform};
# pub struct SimpleButtonSystem {
#   reader_id: ReaderId<UiEvent>,
# }
# 
# impl System for SimpleButtonSystem {
#   type SystemData = (
#       .read_resource::<EventChannel<UiEvent>>(),
#     .read_component::<UiTransform>(),
#       .write_component::<UiText>()
#   );
# 
    fn run(&mut self, (events, transforms, mut texts): Self::SystemData) {
        for event in events.read(&mut self.reader_id) {
            let button_text = texts.get_mut(event.target).unwrap();

            match event.event_type {
                UiEventType::HoverStart => {
                    button_text.color = [1.0, 1.0, 1.0, 1.0];
                }
                UiEventType::HoverStop => {
                    button_text.color = [1.0, 1.0, 1.0, 0.5];
                }
                _ => {}
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

[eventchannel]: https://specs.amethyst.rs/docs/api/shrev/struct.eventchannel
[readerid]: https://docs.rs/specs/~0.16/specs/struct.ReaderId.html
[sys_ini]: ../concepts/system/system_initialization.html
[uievent]: https://docs.amethyst.rs/master/amethyst_ui/struct.UiEvent.html
