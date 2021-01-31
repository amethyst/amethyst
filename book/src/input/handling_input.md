# Handling Input

Amethyst uses an `InputHandler` to handle user input.
You initialize this `InputHandler` by creating an `InputBundle` and adding it to the game data.

```rust
use amethyst::{input::InputBundle, prelude::*};

# struct Example;
# impl SimpleState for Example {}
# fn main() -> amethyst::Result<()> {

    let input_bundle = InputBundle::new();

    let mut world = World::default();
    let game_data = DispatcherBuilder::default()
        //..
        .add_bundle(input_bundle)?;

#   Ok(())
# }
```

To use the `InputHandler` inside a `System` you have to add it to the `SystemData`. With this you can check for events from input devices.

```rust
use amethyst::{
    ecs::{System, World},
    input::{ControllerButton, InputHandler, VirtualKeyCode},
    prelude::*,
};

struct ExampleSystem;

impl System for ExampleSystem {
    type SystemData = .read_resource::<InputHandler>();

    fn run(&mut self, input: Self::SystemData) {
        // Gets mouse coordinates
        if let Some((x, y)) = input.mouse_position() {
            //..
        }

        // Gets all connected controllers
        let controllers = input.connected_controllers();
        for controller in controllers {
            // Checks if the A button is down on each connected controller
            let buttonA = input.controller_button_is_down(controller, ControllerButton::A);
            //..
        }

        // Checks if the A button is down on the keyboard
        let buttonA = input.key_is_down(VirtualKeyCode::A);
        //..
    }
}
```

You can find all the methods from `InputHandler` [here][input_ha].

Now you have to add the `System` to the game data, like you would do with any other `System`. A `System` that uses an `InputHandler` needs `"input_system"` inside its dependencies.

```rust
# use amethyst::{ecs::*, prelude::*};
# struct ExampleSystem;
# impl System for ExampleSystem {
#   fn run(&mut self, _: ()) {}
# }
# fn main() {
    let game_data = DispatcherBuilder::default()
        //..
        .with(ExampleSystem, "example_system", &["input_system"]);
    //..
# }
```

## Defining Key Bindings in a File

Instead of hard coding in all the key bindings, you can store all the bindings in a config file. A config file for key bindings with the RON format looks something like this:

```ron
(
    axes: {
        "vertical": Emulated(pos: Key(W), neg: Key(S)),
        "horizontal": Emulated(pos: Key(D), neg: Key(A)),
    },
    actions: {
        "shoot": [[Key(Space)]],
    },
)
```

The axis values range from `-1.0` to `1.0`. For an `Emulated` axis controller such as keyboard buttons, the values are distinct:

- `0.0` when neither, or both the `neg` or `pos` buttons are pressed.
- `-1.0` when the `neg` button is pressed.
- `1.0` when the `pos` button is pressed.

Values between `0.0` and `1.0` are possible when using a controller such as a joystick. This can be enabled via the `"sdl_controller"` feature.

The action is a boolean, which is set to true when the buttons are pressed. The action binding is defined by a two-level array:

- The inner array specifies the buttons that must be pressed at the same time to send the action.
- The outer array specifies different combinations of those buttons that send the action.

The possible inputs you can specify for axes are listed [here][in_axis]. The possible inputs you can specify for actions are listed [here][button].

To add these bindings to the `InputBundle` you simply need to call the `with_bindings_from_file` function on the `InputBundle`.

```rust
# use amethyst::{input::*, prelude::*, utils::*};
# fn main() -> amethyst::Result<()> {
    let root = application_root_dir()?;
    let bindings_config = root.join("config").join("bindings.ron");

    let input_bundle = InputBundle::new().with_bindings_from_file(bindings_config)?;

    //..
#   Ok(())
# }
```

And now you can get the [axis][axis_val] and [action][is_down] values from the `InputHandler`.

```rust
use amethyst::{
    core::{Transform},
    ecs::{System, World},
    input::InputHandler,
    prelude::*,
};

struct Player {
    id: usize,
}

impl Player {
    pub fn shoot(&self) {
        println!("PEW! {}", self.id);
    }
}

struct MovementSystem;

impl System for MovementSystem {
    type SystemData = (
        .write_component::<Transform>().read_component::<Player>(),
        .read_resource::<InputHandler>(),
    );

    fn run(&mut self, (mut transforms, players, input): Self::SystemData) {
        for (player, transform) in (&players, &mut transforms).join() {
            let horizontal = input.axis_value("horizontal").unwrap_or(0.0);
            let vertical = input.axis_value("vertical").unwrap_or(0.0);

            let shoot = input.action_is_down("shoot").unwrap_or(false);

            transform.move_up(horizontal);
            transform.move_right(vertical);

            if shoot {
                player.shoot();
            }
        }
    }
}
```

[axis_val]: https://docs.amethyst.rs/master/amethyst_input/struct.InputHandler.html#method.axis_value
[button]: https://docs.amethyst.rs/master/amethyst_input/enum.Button.html
[input_ha]: https://docs.amethyst.rs/master/amethyst_input/struct.InputHandler.html#methods
[in_axis]: https://docs.amethyst.rs/master/amethyst_input/enum.Axis.html
[is_down]: https://docs.amethyst.rs/master/amethyst_input/struct.InputHandler.html#method.action_is_down
