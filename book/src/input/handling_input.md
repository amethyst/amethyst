# Handling Input

Amethyst uses an `InputHandler` to handle user input.
You initialise this `InputHandler` by creating an `InputBundle` and adding it to the game data.

```rust,edition2019,no_run,noplaypen
use amethyst{
  prelude::*,
  utils::application_root_dir,
  input::{InputBundle, StringBindings},
};

fn main() -> amethyst::Result<()> {
  // StringBindings is the default BindingTypes
  let input_bundle = InputBundle::<StringBindings>::new();

  let game_data = GameDataBuilder::default()
    //..
    .with_bundle(input_bundle)?
    //..

  //..
}
```

To use the `InputHandler` inside a `System` you have to add it to the `SystemData`. With this you can check for events from input devices.

```rust,edition2019,no_run,noplaypen
use amethyst{
  prelude::*,
  input::{InputHandler, ControllerButton, VirtualKeyCode, StringBindings},
  ecs::{Read, System};
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
      let buttonA = input.controller_button_is_down(controller, ControllerButton::A);
      //..
    }

    // Checks if the A button is down on the keyboard
    let buttonA = input.key_is_down(VirtualKeyCode::A);
    //..
  }
}
```

You can find all the methods from `InputHandler` [here](https://docs-src.amethyst.rs/stable/amethyst_input/struct.InputHandler.html#methods).

Now you have to add the system to the game date, just like you would add any other `System`. A `System` that uses an `InputHandler` needs "input_system" inside its dependencies.

```rust,edition2019,no_run,noplaypen
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
    "shoot": [[Key(Space)]]
  },
)
```

The axes are values from -1, 0 or 1 depending on the state. If the input specified for pos is triggered the value is 1, if the input for neg is triggered the value is -1, by default the value is 0.

The action is a simple boolean, which is of course set to true if the specified input is triggered. The action binding is inside two arrays, this is because an action can be triggered by a combination of inputs, which is the purpose of the inner array. Each action can have multiple inputs or combinations of inputs that trigger it, which is the purpose of the outer array.

The possible inputs you can specify for axis are listed [here](https://docs-src.amethyst.rs/stable/amethyst_input/enum.Axis.html). The possible inputs you can specify for actions are listed [here](https://docs-src.amethyst.rs/stable/amethyst_input/enum.Button.html).

To add these bindings to the `InputBundle` you simply need to call the `with_bindings_from_file` function on the InputBundle.

```rust,edition2019,no_run,noplaypen
  let root = application_root_dir()?;
  let bindings_config = root.join("resources").join("bindings_config.ron");

  let input_bundle = 
    InputBundle::<StringBindings>::new()
    .with_bindings_from_file(bindings_config)?;

  //..
}
```

And now you can get the [axis](https://docs-src.amethyst.rs/stable/amethyst_input/struct.InputHandler.html#method.axis_value) and [action](https://docs-src.amethyst.rs/stable/amethyst_input/struct.InputHandler.html#method.action_is_down) values from the `InputHandler`.

```rust,edition2019,no_run,noplaypen
use amethyst{
  prelude::*,
  core::Transform;
  ecs::{Join, Read, ReadStorage, System, WriteStorage};
  input::{InputHandler, StringBindings},
};

struct MovementSystem;

impl<'s> System<'s> for MovementSystem {
  type SystemData = (
    WriteStorage<'s, Transform>,
    ReadStorage<'s, Player>,
    Read<'s, InputHandler<StringBindings>>,
  );

  fn run(&mut self, (mut transform, mut player, input): Self::SystemData) {
    for (player, transform) in (&mut player, &mut transform).join() {
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
