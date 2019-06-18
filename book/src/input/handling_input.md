# Handling Input

Amethyst uses an `InputHandler` to handle user input.
You initialise this `InputHandler` by creating an `InputBundle` and adding it to the game data.

```rust,no_run,noplaypen
# extern crate amethyst;
use amethyst::{
    prelude::*,
    input::{InputBundle, StringBindings},
#   window::{WindowBundle, DisplayConfig},
};

# struct Example;
# impl SimpleState for Example {}
fn main() -> amethyst::Result<()> {
    // StringBindings is the default BindingTypes
    let input_bundle = InputBundle::<StringBindings>::new();

    let game_data = GameDataBuilder::default()
        //..
        # .with_bundle(WindowBundle::from_config(DisplayConfig::default()))?
        .with_bundle(input_bundle)?
        # ;
        //..

    //..
    # let mut game = Application::new("./", Example, game_data)?;
    # game.run();
    # Ok(())
}
```

To use the `InputHandler` inside a `System` you have to add it to the `SystemData`. With this you can check for events from input devices.

```rust,no_run,noplaypen
# extern crate amethyst;
use amethyst::{
  prelude::*,
  input::{InputHandler, ControllerButton, VirtualKeyCode, StringBindings},
  ecs::{Read, System},
};

struct ExampleSystem;

impl<'s> System<'s> for ExampleSystem {
    // The same BindingTypes from the InputBundle needs to be inside the InputHandler
    type SystemData = Read<'s, InputHandler<StringBindings>>;

    fn run(&mut self, input: Self::SystemData) {
        // Gets mouse coordinates
        if let Some((x, y)) = input.mouse_position() {
            //..
        }

        // Gets all connected controllers
        let controllers = input.connected_controllers();
        for controller in controllers {
            // Checks if the A button is down on each connected controller
            let button_a = 
                input
                .controller_button_is_down(controller, ControllerButton::A);
            //..
        }

        // Checks if the A button is down on the keyboard
        let button_a = input.key_is_down(VirtualKeyCode::A);
        //..
    }
}
```

You can find all the methods from `InputHandler` [here](https://docs-src.amethyst.rs/stable/amethyst_input/struct.InputHandler.html#methods).

Now you have to add the `System` to the game data, just like you would do with any other `System`. A `System` that uses an `InputHandler` needs `"input_system"` inside its dependencies.

```rust,ignore
    let game_data = GameDataBuilder::default()
        //..
        .with(ExampleSystem, "example_system", &["input_system"])
        //..
```

## Defining Key Bindings in a File

Instead of hard coding in all the key bindings, you can store all the bindings in a config file. A config file for key bindings with the RON format looks something like this:

```ron,ignore
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

* `0.0` when neither, or both the `neg` or `pos` buttons are pressed.
* `-1.0` when the `neg` button is pressed.
* `1.0` when the `pos` button is pressed.

Values between `0.0` and `1.0` are possible when using a controller such as a joystick. This can be enabled via the `"sdl_controller"` feature.

The action is a boolean, which is set to true when the buttons are pressed. The action binding is defined by a two-level array:

* The inner array specifies the buttons that must be pressed at the same time to send the action.
* The outer array specifies different combinations of those buttons that send the action.

The possible inputs you can specify for axes are listed [here](https://docs-src.amethyst.rs/stable/amethyst_input/enum.Axis.html). The possible inputs you can specify for actions are listed [here](https://docs-src.amethyst.rs/stable/amethyst_input/enum.Button.html).

To add these bindings to the `InputBundle` you simply need to call the `with_bindings_from_file` function on the `InputBundle`.

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::{
#   prelude::*,
#   utils::application_root_dir,
#   input::{InputBundle, StringBindings},
#   window::{WindowBundle, DisplayConfig},
# };
# struct Example;
# impl SimpleState for Example {}
# fn main() -> amethyst::Result<()> {
    let root = application_root_dir()?;
    let bindings_config = root.join("resources").join("bindings_config.ron");

    let input_bundle = 
        InputBundle::<StringBindings>::new()
        .with_bindings_from_file(bindings_config)?;

    //..
    # let game_data = GameDataBuilder::default()
    #     .with_bundle(WindowBundle::from_config(DisplayConfig::default()))?
    #     .with_bundle(input_bundle)?;
    # let mut game = Application::new("./", Example, game_data)?;
    # game.run();
    # Ok(())
# }
```

And now you can get the [axis](https://docs-src.amethyst.rs/stable/amethyst_input/struct.InputHandler.html#method.axis_value) and [action](https://docs-src.amethyst.rs/stable/amethyst_input/struct.InputHandler.html#method.action_is_down) values from the `InputHandler`.

```rust,no_run,noplaypen
# extern crate amethyst;
use amethyst::{
    prelude::*,
    core::Transform,
    ecs::{Join, Read, ReadStorage, System, WriteStorage, DenseVecStorage, Component},
    input::{InputHandler, StringBindings},
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

struct MovementSystem;

impl<'s> System<'s> for MovementSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Player>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (mut transform, player, input): Self::SystemData) {
        for (player, transform) in (&player, &mut transform).join() {
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
