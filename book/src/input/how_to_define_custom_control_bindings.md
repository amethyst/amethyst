# How to Define Custom Control Bindings

Instead of using `StringBindings` for an `InputBundle` you probably want to use a custom type in production, as `StringBindings` are mainly meant to be used for prototyping and not very efficient.

Using a custom type to handle input instead of using `String` has many advantages:

* A `String` uses quite a lot of memory compared to something like an enum.
* Inputting a `String` when retrieving input data is error-prone if you mistype it or change the name.
* A custom type can hold additional information.

## Defining Custom Input `BindingTypes`

Defining a custom type for the `InputBundle` is done by implementing the `BindingTypes` trait. This trait contains two types, an `Axis` type and an `Action` type. These types are usually defined as enums.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# extern crate serde;
use std::fmt::{self, Display};

use amethyst::input::{BindingTypes, Bindings};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
enum AxisBinding {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
enum ActionBinding {
    Shoot,
}

impl Display for AxisBinding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for ActionBinding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
struct MovementBindingTypes;

impl BindingTypes for MovementBindingTypes {
    type Axis = AxisBinding;
    type Action = ActionBinding;
}
```

The `Axis` and `Action` type both need to derive all the traits listed above, the first five are used by Amethyst and the last two are for reading and writing to files correctly. They also need to implement `Display` if you want to add a bindings config file.

For serializing and deserializing you need to add [serde] to the dependencies like this:

```toml,ignore
serde = { version = "1", features = ["derive"] }
```

If you want to add additional information you can add it to the enum or change the `Axis` and `Action` types to a struct. For example:

```rust,ignore
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
enum AxisBinding {
    Horizontal(usize),
    Vertical(usize),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
enum ActionBinding {
    Shoot(usize),
}

//..
```

We can now use this custom type in our `InputBundle` and create a RON config file for our bindings.

The config file might look something like this:

```ron,ignore
(
    axes: {
        Vertical(0): Emulated(pos: Key(W), neg: Key(S)),
        Horizontal(0): Emulated(pos: Key(D), neg: Key(A)),
        Vertical(1): Emulated(pos: Key(Up), neg: Key(Down)),
        Horizontal(1): Emulated(pos: Key(Right), neg: Key(Left)),
    },
    actions: {
        Shoot(0): [[Key(Space)]],
        Shoot(1): [[Key(Return)]],
    },
)
```

Here the number after the binding type could be the ID of the player, but you can supply any other data as long as it derives the right traits.

With the config file we can create an `InputBundle` like in the previous section.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::input::StringBindings as MovementBindingTypes;
use amethyst::input::InputBundle;

# fn main() -> amethyst::Result<()> {
#
# let input_config = "input.ron";
#
let input_bundle = 
    InputBundle::<MovementBindingTypes>::new()
        .with_bindings_from_file(input_config)?;
#
# Ok(())
# }
```

And add the `InputBundle` to the game data just like before.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# use amethyst::input::{InputBundle, StringBindings};
#
# fn main() -> amethyst::Result<()> {
# let input_bundle = InputBundle::<StringBindings>::default();
#
let mut world = World::default();
let game_data = DispatcherBuilder::default()
    //..
    .with_bundle(input_bundle)?
    //..
#   ;
#
# Ok(())
# }
```

## Using the `InputHandler` with a Custom `BindingTypes`

Now that we have added an `InputBundle` with a custom `BindingTypes`, we can use the `InputHandler` just like with `StringBindings`, but instead of using `String`s we use our custom enums.

```rust,edition2018,no_run,noplaypen,ignore
# extern crate amethyst;
use amethyst::{
    core::{Transform, SystemDesc},
    derive::SystemDesc,
    ecs::{Component, DenseVecStorage, Join, Read, ReadStorage, System, SystemData, World, WriteStorage},
    input::{AxisBinding, InputHandler},
};

struct Player {
    id: usize,
}

impl Player {
    pub fn shoot(&self) {
        println!("PEW! {}", self.id);
    }
}

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}

#[derive(SystemDesc)]
struct MovementSystem;

impl<'s> System<'s> for MovementSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Player>,
        Read<'s, InputHandler<MovementBindingTypes>>,
    );

    fn run(&mut self, (mut transform, player, input): Self::SystemData) {
        for (player, transform) in (&player, &mut transform).join() {
            let horizontal = input
                .axis_value(&AxisBinding::Horizontal(player.id))
                .unwrap_or(0.0);
            let vertical = input
                .axis_value(&AxisBinding::Vertical(player.id))
                .unwrap_or(0.0);
            
            let shoot = input
                .action_is_down(&ActionBinding::Shoot(player.id))
                .unwrap_or(false);
            
            transform.move_up(horizontal);
            transform.move_right(vertical);
            
            if shoot {
                player.shoot();
            }
        }
    }
}
```

And don't forget to add the `MovementSystem` to the game data.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# use amethyst::ecs::*;
# use amethyst::core::SystemDesc;
# use amethyst::derive::SystemDesc;
# #[derive(SystemDesc)]
# struct MovementSystem;
# impl<'a> System<'a> for MovementSystem {type SystemData=(); fn run(&mut self, _: ()) {}}
let game_data = DispatcherBuilder::default()
//..
    .with(MovementSystem, "movement_system", &["input_system"])
//..
#   ;
```

[serde]: https://crates.io/crates/serde