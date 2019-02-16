# Creating a window

Let's start a new project:

`amethyst new pong`

If you run this project with `cargo run`, you'll end up with a window titled
"pong" that renders a really delightful shade of green. If you're having trouble 
getting the project to run, double check the [Getting Started][gs] guide.

We've created and opened a window, so we're basically done! But let's write this
functionality ourselves so we're sure we know what's going on. Close the window
we just opened before reading on.

**In `src` there's a `main.rs` file. Delete everything in that file, then
add these imports:**

```rust,edition2018,no_run,noplaypen
extern crate amethyst;

use amethyst::prelude::*;
use amethyst::renderer::{DisplayConfig, DrawFlat2D, Event, Pipeline,
                         RenderBundle, Stage, VirtualKeyCode};
```

We'll be learning more about these as we go through this tutorial. The prelude
includes the basic (and most important) types like `Application`, `World`, and
`State`.

Now we create our core game struct:

```rust,edition2018,no_run,noplaypen
pub struct Pong;
```

We'll be implementing the [`SimpleState`][st] trait on this struct, which is used by
Amethyst's state machine to start, stop, and update the game. But for now we'll
just implement two methods:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::input::{is_close_requested, is_key_down};
# use amethyst::prelude::*;
# use amethyst::renderer::{DisplayConfig, DrawFlat, Pipeline,
#                          PosTex, RenderBundle, Stage};
# struct Pong;
impl SimpleState for Pong {
}
```

The `SimpleState` already implements a bunch of stuff for us, like the `update` 
and `handle_event` methods that you would have to implement yourself were you 
using just a regular `State`. In particular, the default implementation for
`handle_event` returns `Trans::Quit` when a close signal is received
from your operating system, like when you press the close button in your graphical
environment. This allows the application to quit as needed. The default 
implementation for `update` then just returns `Trans::None`, signifying that
nothing is supposed to happen.

Now that we know we can quit, let's add some code to actually get things
started! We'll start with our `main` function, and we'll have it return a
`Result` so that we can use `?`. This will allow us to automatically exit
if any errors occur during setup.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
fn main() -> amethyst::Result<()> {

    // We'll put the rest of the code here.

    Ok(())
}
```

Inside `main()` we first start the amethyst logger with a default `LoggerConfig`
so we can see errors, warnings and debug messages while the program is running.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# fn main() -> amethyst::Result<()> {
amethyst::start_logger(Default::default());
# Ok(())
# }
```

After the logger is started, we need to create a `DisplayConfig` to store 
the configuration for our game's display. We can either define the configuration in
our code, or better yet load it from a file. The latter approach is handier, as 
it allows us to change configuration (e.g, the display size) without having to 
recompile our game every time.

Starting the project with `amethyst new` should have automatically generated 
`DisplayConfig` data in `resources/display_config.ron`.
If you created the project manually, go ahead and create those now.

In either case, open `display_config.ron` and change its contents to the following:

```rust,ignore
(
  title: "Pong!",
  dimensions: Some((500, 500)),
  max_dimensions: None,
  min_dimensions: None,
  fullscreen: false,
  multisampling: 0,
  visibility: true,
  vsync: true,
)
```

> If you have never run into Rusty Object Notation before (or RON for short), 
> it is a data storage format that mirrors Rust's syntax. Here, the
> data represents the [`DisplayConfig`][displayconf] struct. If you want to
> learn more abour the RON syntax, you can visit the [official repository][ron].

This will set the default window dimensions to 500 x 500, and make the title bar
say "Pong!" instead of the sad, lowercase default of "pong".

In `main()` in `main.rs`, we will load the configuration from the file:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# use amethyst::renderer::DisplayConfig;
# fn main() -> amethyst::Result<()> {
use amethyst::utils::application_dir;

let path = application_dir("resources/display_config.ron")?;

let config = DisplayConfig::load(&path);
# Ok(())
# }
```

Now, let's copy and paste some rendering code so we can keep moving. 
We'll cover rendering in more depth later in this tutorial.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::renderer::{Pipeline, DrawFlat, PosTex, Stage, DrawFlat2D};
# fn main() {
let pipe = Pipeline::build()
    .with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawFlat2D::new()),
    );
# }
```

The important thing to know right now is that this renders a black background.
If you want a different color you can tweak the RGBA values inside the
`.clear_target` method. Values range from 0.0 to 1.0, so to get that cool green
color we started with back then, for instance, you can try
`[0.00196, 0.23726, 0.21765, 1.0]`.

Now let's pack everything up and run it:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# use amethyst::renderer::{DisplayConfig, DrawFlat2D, Pipeline,
#                        RenderBundle, Stage};
# fn main() -> amethyst::Result<()> {
# let path = "./resources/display_config.ron";
# let config = DisplayConfig::load(&path);
# struct Pong;
# impl SimpleState for Pong { }
let pipe = Pipeline::build()
    .with_stage(
#        Stage::with_backbuffer()
#          .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
#          .with_pass(DrawFlat2D::new()),
    // --snip--
    );

let game_data = GameDataBuilder::default()
    .with_bundle(
      RenderBundle::new(pipe, Some(config))
        .with_sprite_sheet_processor()
    )?;

let mut game = Application::new("./", Pong, game_data)?;

game.run();
# Ok(())
# }
```

We've discovered Amethyst's root object: [Application][ap]. It binds the OS
event loop, state machines, timers and other core components in a central place.
Here we're creating a new `RenderBundle`, adding the `Pipeline` we created,
along with our config, and building. There is also a helper function
`with_basic_renderer` on `GameDataBuilder` that you can use to create your
`Pipeline` and `RenderBundle`, that performs most of the actions above. This
function is used in the full `pong` example in the `Amethyst` repository.

Then we call `.run()` on `game` which begins the gameloop. The game will
continue to run until our `SimpleState` returns `Trans::Quit`, or when all states
have been popped off the state machine's stack.

Success! Now we should be able to compile and run this code and get a window.
It should look something like this:

![Step one](../images/pong_tutorial/pong_01.png)


[ron]: https://github.com/ron-rs/ron
[st]: https://www.amethyst.rs/doc/latest/doc/amethyst/prelude/trait.SimpleState.html
[ap]: https://www.amethyst.rs/doc/latest/doc/amethyst/struct.Application.html
[gs]: ../getting-started.html
[displayconf]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/struct.DisplayConfig.html

