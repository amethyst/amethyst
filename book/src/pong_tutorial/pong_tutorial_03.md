# Moving the paddles

This is a big step. After this we'll be able to move our entities around, using 
the keyboard. So, basically we'll have a video game. The good news is the 
structure we have in place so far makes adding this functionality relatively 
straightforward.

First let's make a new file called `paddle.rs` which will hold our 
`PaddleSystem` struct and implementation. Remember how ECS stands for "Entity-
component system"? Well, this is the "system" part. Systems iterate through 
components and *do stuff*.

Let's import what we'll need and declare our `PaddleSystem` struct:

```rust,ignore
use pong::{Paddle, Side};

use amethyst::core::timing::Time;
use amethyst::core::transform::LocalTransform;
use amethyst::ecs::{Fetch, Join, System, WriteStorage};
use amethyst::input::InputHandler;

pub struct PaddleSystem;
```

Now we're going to implement the `System` trait for `PaddleSystem`. This 
involves the Rust concept of lifetimes, which you might have managed to avoid in 
your Rust learning journey thus far. Don't be scared! 
[Here's the relevant part of the The Rust Book][lifetimes], if you want a 
refresher. But basically, all we're doing is saying this method has a lifetime 
`s`, and all the data this method relies on needs to live as long as `s`. This 
is how we can run multiple systems in parallel without fear.

```rust,ignore
impl<'s> System<'s> for PaddleSystem {
    //We'll define the system in here
}
```

First off we need to define the `SystemData`, which is a tuple of items which 
each implement `SystemData`, such as `WriteStorage`, `ReadStorage`, and `Fetch`. 

```rust,ignore
type SystemData = (
    WriteStorage<'s, Paddle>,
    WriteStorage<'s, LocalTransform>,
    Fetch<'s, Time>,
    Fetch<'s, InputHandler<String, String>>,
);
```

Now we're done worrying about lifetimes! Painless, right? Also, you might notice 
that this `SystemData` tuple is a nice and succinct description of what we want 
to do with this `System`: we want to change the `LocalTransform` of a `Paddle` 
over `Time` based on keyboard `Input`.

`Time` is a resource that's included by the engine automatically, but we don't 
have `InputHandler` yet. We'll fix that soon! 

First we're going to make a few small tweaks to `pong.rs`. We need to know how 
fast to move the paddles over time, so let's add a velocity constant:

```rust,ignore
const PADDLE_VELOCITY: f32 = 1.0;
```

We also need to make our `Side` enum and `Paddle` struct public, and add the 
velocity to the `Paddle`. Let's also fix our `new` method to actually use our 
constants:

```rust,ignore
#[derive(PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

pub struct Paddle {
    pub side: Side,
    pub width: f32,
    pub height: f32,
    pub velocity: f32,
}

impl Paddle {
    fn new(side: Side) -> Paddle {
        Paddle {
            side: side,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
            velocity: PADDLE_VELOCITY
        }
    }
}
```

Okay, now for that mysterious `InputHandler`. In `main.rs` we can import the `InputBundle`, which includes the `InputHandler` 
resource.

```rust,ignore
use amethyst::input::InputBundle;
```

Inside the `run()` function we'll make a few changes. First we want to load an 
`input.ron` file which will define our keybindings, and we'll also rename our 
`display_config` import for clarity:

```rust,ignore
let display_config = "./resources/display_config.ron";
let key_bindings_path = "./resources/input.ron";

let config = DisplayConfig::load(&display_config);
```

In your project's `resources` folder make a file called `input.ron` and define 
some keybindings in there:

```rust,ignore
(
    axes: {
        "left_paddle": (
            pos: Key(W),
            neg: Key(S),
        ),
        "right_paddle": (
            pos: Key(Up),
            neg: Key(Down),
        ),
    },
    actions: {
    },
)
```

And now we add the `InputBundle` to our `Application`. While we're here, let's 
also add our `PaddleSystem`:

```rust,ignore
let mut game = Application::build("./", Pong)?
    .with_bundle(
        InputBundle::<String, String>::new()
        .with_bindings_from_file(&key_bindings_path)
    )?
    .with::<PaddleSystem>(PaddleSystem, "paddle_system", &["input_system"])
    .with_bundle(TransformBundle::new())?
    .with_bundle(RenderBundle::new())?
    .with_local(RenderSystem::build(pipe, Some(config))?)
    .build()?;
```

When we add the `PaddleSystem`, we're saying that we want to add it with the 
name of `"paddle_system"` and that it also has a *dependency* on
`"input_system"`. 
That `"input_system"` name was defined by the `InputBundle`. Declaring 
`"paddle_system"` to be dependent on `"input_system"` allows our systems to run 
in parallel when possible, while still making sure work is done in order. 
`"paddle_system"` (which moves the paddles around) will only run once 
`"input_system"` is done (which is where we receive the keyboard inputs to know 
where the paddles should go).

Okay, so now we've declared a system and added it to our game. Let's make it 
actually do something!

Back in `paddle.rs` we'll define a `run()` method in our `System` 
implementation:

```rust,ignore
fn run(&mut self, (mut paddles, mut transforms, time, input): Self::SystemData) {

    //Do stuff!

}
```

If that function signature looks strange to you, it's because we're using 
destructuring to give names to the items in our `SystemData` tuple.

Inside the `run()` function let's finally write the behavior we want for our 
paddles:

```rust,ignore
// Iterate over all planks and move them according to the input the user
// provided.
for (paddle, transform) in (&mut paddles, &mut transforms).join() {
    let opt_movement = match paddle.side {
        Side::Left => input.axis_value("left_paddle"),
        Side::Right => input.axis_value("right_paddle"),
    };

    if let Some(movement) = opt_movement {
        transform.translation.y +=
            paddle.velocity * time.delta_seconds() * movement as f32;

        // We make sure the paddle remains in the arena.
        transform.translation.y = transform.translation.y
            .max(-1.0)
            .min(1.0 - paddle.height);
    }
}
```

`axis_value("name")` is a method provided by the `InputHandler`. The API docs 
include a [whole list of these methods][input].

The real heart of this code is the mutation of `transform.translation.y`.
`transform.translation` is a `Vector3`, and we can access its members by name 
(`.x`,`.y`,`.z`) or position (`[0]`,`[1]`,`[2]`). The `.max` and `.min` methods 
are just regular `f32` methods which compare an `f32` value to `self`.

And we're done with this step! Compile and run and you should be able to move 
the left paddle with W and S, and the right paddle with the Up and Down arrow 
keys.

[lifetimes]: https://doc.rust-lang.org/book/second-edition/ch10-03-lifetime-syntax.html
[input]: https://www.amethyst.rs/doc/develop/doc/amethyst_input/struct.InputHandler.html#method.axis_value