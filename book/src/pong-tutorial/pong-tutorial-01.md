# Setting up the project

In this chapter, we will go through the basics of setting up the amethyst project,
starting the logger, opening a window and preparing a simple rendering setup.

## Creating a new project

Let's start a new project:

`amethyst new pong`

Update the dependencies in the project's `Cargo.toml` so that it contains:

```toml
[package]
name = "pong"
version = "0.1.0"
authors = []
edition = "2018"

[dependencies.amethyst]
version = "0.11"
features = ["vulkan"]
```

Alternatively, if you are developing on macOS, you might want to use the `metal` rendering backend instead of `vulkan`. In this case, you should change the `features` entry in the `amethyst` dependency table.

```toml
[dependencies.amethyst]
version = "0.11"
features = ["metal"]
```

We can start with editing the `main.rs` file inside `src` directory.
You can delete everything in that file, then add these imports:

```rust,ignore
//! Pong Tutorial 1

use amethyst::{
    assets::Processor,
    ecs::{ReadExpect, Resources, SystemData},
    prelude::*,
    renderer::{
        pass::DrawFlat2DDesc, types::DefaultBackend, Factory, Format, GraphBuilder, GraphCreator,
        Kind, RenderGroupDesc, RenderingSystem, SpriteSheet, SubpassBuilder,
    },
    utils::application_root_dir,
    window::{ScreenDimensions, Window, WindowBundle},
};
```

We'll be learning more about these as we go through this tutorial. The prelude
includes the basic (and most important) types like `Application`, `World`, and
`State`. We also import all the necessary types to define a basic rendering pipeline.

Now we have all the dependencies installed and imports prepared, we are ready to start
working on defining our game code.

## Creating the game state

Now we create our core game struct:

```rust,edition2018,no_run,noplaypen
pub struct Pong;
```

We'll be implementing the [`SimpleState`][simplestate] trait on this struct, which is
used by Amethyst's state machine to start, stop, and update the game.

```rust,ignore
impl SimpleState for Pong {}
```

Implementing the `SimpleState` teaches our application what to do when a close signal
is received from your operating system. This happens when you press the close
button in your graphical environment. This allows the application to quit as needed.

Now that our `Pong` is already a game state, let's add some code to actually get things
started! We'll start with our `main()` function, and we'll have it return a
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

> **Note:** The [SimpleState][simplestate] is just a simplified version of [State][state] trait.
> It already implements a bunch of stuff for us, like the `State`'s `update`
> and `handle_event` methods that you would have to implement yourself were you
> using just a regular `State`. Its behavior mostly cares about handling the exit signal cleanly,
> by just quitting the application directly from the current state.

## Setting up the logger

Inside `main()` we first start the amethyst logger with a default `LoggerConfig`
so we can see errors, warnings and debug messages while the program is running.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# fn main() {
amethyst::start_logger(Default::default());
# }
```

From now on, every info, warning, and error will be present and clearly formatted
inside your terminal window.


> **Note:** There are many ways to configure that logger, for example, to write the
> log to the filesystem. You can find more information about how to do that in [Logger API
> reference][log].
> We will use the most basic setup in this tutorial for simplicity.

## Preparing the display config

Next, we need to create a `DisplayConfig` to store the configuration for our game's
window. We can either define the configuration in our code or better yet load it
from a file. The latter approach is handier, as it allows us to change configuration
(e.g, the window size) without having to recompile our game every time.

Starting the project with `amethyst new` should have automatically generated 
`DisplayConfig` data in `config/display.ron`. If you created the
project manually, go ahead and create it now.

In either case, open `display.ron` and change its contents to the
following:

```rust,ignore
(
    title: "Pong!",
    dimensions: Some((500, 500)),
)
```

> **Note:** If you have never run into Rusty Object Notation before (or RON for short), 
> it is a data storage format that mirrors Rust's syntax. Here, the
> data represents the [`DisplayConfig`][displayconf] struct. If you want to
> learn more about the RON syntax, you can visit the [official repository][ron].

This will set the default window dimensions to 500 x 500, and make the title bar
say "Pong!" instead of the sad, lowercase default of "pong".

In `main()` in `main.rs`, we will prepare the path to a file containing
the display configuration:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::{
#     utils::application_root_dir,
#     Error,
# };
#
# fn main() -> Result<(), Error>{
let app_root = application_root_dir()?;
let display_config_path = app_root.join("config").join("display.ron");
#     Ok(())
# }
```

## Opening a window

After preparing the display config, it's time to actually use it. To do that,
we have to create an amethyst application scaffolding and tell it to open a window for us.

In `main()` in `main.rs` we are going to add the basic application setup:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::{prelude::*, window::*};
# fn main() -> Result<(), amethyst::Error>{
# let display_config_path = "";
# struct Pong; impl SimpleState for Pong {}
let game_data = GameDataBuilder::default()
    // The WindowBundle provides all the scaffolding for opening a window
    .with_bundle(WindowBundle::from_config_path(display_config_path))?;

# let app_root = std::path::PathBuf::from(".");
let assets_dir = app_root.join("assets");
let mut game = Application::new(assets_dir, Pong, game_data)?;
game.run();
#     Ok(())
# }
```

Here we're creating a new `WindowBundle` that uses the config we prepared above.
That bundle is being used as a part of `GameDataBuilder`, a central repository
of all the game logic that runs periodically during the game runtime.

> **Note:** We will cover systems and bundles in more details later, for now, think of the
> bundle as a group of functionality that together provides a certain feature to the engine.
> You will surely be writing your own bundles for your own game's features soon.

That builder is then combined with the game state struct (`Pong`), creating the overarching
Amethyst's root object: [Application][ap]. It binds the OS event loop, state machines,
timers and other core components in a central place.

Then we call `.run()` on `game` which starts the game loop. The game will
continue to run until our `SimpleState` returns `Trans::Quit`, or when all states
have been popped off the state machine's stack.

Try compiling the code now. You should be able to see the window already.
The content of that window right now is undefined and up to the operating system.
It's time to start drawing on it.

## Setting up basic rendering

Now, let's define some rendering code so we can keep moving. This part is not strictly
necessary to show a window, but we need the renderer to display anything inside it.

We'll cover rendering in more depth later in this tutorial, but for now place the
following code _below_ the `main()` function:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::{
#     assets::Processor,
#     ecs::{ReadExpect, Resources, SystemData},
#     prelude::*,
#     renderer::{
#         pass::DrawFlat2DDesc, types::DefaultBackend, Factory, Format, GraphBuilder, GraphCreator,
#         Kind, RenderGroupDesc, RenderingSystem, SpriteSheet, SubpassBuilder,
#     },
#     utils::application_root_dir,
#     window::{ScreenDimensions, Window, WindowBundle},
# };
// This graph structure is used for creating a proper `RenderGraph` for rendering.
// A renderGraph can be thought of as the stages during a render pass. In our case,
// we are only executing one subpass (DrawFlat2D, or the sprite pass). This graph
// also needs to be rebuilt whenever the window is resized, so the boilerplate code
// for that operation is also here.
#[derive(Default)]
struct ExampleGraph {
    dimensions: Option<ScreenDimensions>,
    dirty: bool,
}

impl GraphCreator<DefaultBackend> for ExampleGraph {
    // This trait method reports to the renderer if the graph must be rebuilt, usually because
    // the window has been resized. This implementation checks the screen size and returns true
    // if it has changed.
    fn rebuild(&mut self, res: &Resources) -> bool {
        // Rebuild when dimensions change, but wait until at least two frames have the same.
        let new_dimensions = res.try_fetch::<ScreenDimensions>();
        use std::ops::Deref;
        if self.dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
            self.dirty = true;
            self.dimensions = new_dimensions.map(|d| d.clone());
            return false;
        }
        return self.dirty;
    }

    // This is the core of a RenderGraph, which is building the actual graph with subpasses and target
    // images.
    fn builder(
        &mut self,
        factory: &mut Factory<DefaultBackend>,
        res: &Resources,
    ) -> GraphBuilder<DefaultBackend, Resources> {
        use amethyst::renderer::rendy::{
            graph::present::PresentNode,
            hal::command::{ClearDepthStencil, ClearValue},
        };

        self.dirty = false;

        // Retrieve a reference to the target window, which is created by the WindowBundle
        let window = <ReadExpect<'_, Window>>::fetch(res);
        let dimensions = self.dimensions.as_ref().unwrap();
        let window_kind = Kind::D2(dimensions.width() as u32, dimensions.height() as u32, 1, 1);

        // Create a new drawing surface in our window
        let surface = factory.create_surface(&window);
        let surface_format = factory.get_surface_format(&surface);

        // Begin building our RenderGraph
        let mut graph_builder = GraphBuilder::new();
        let color = graph_builder.create_image(
            window_kind,
            1,
            surface_format,
            // clear screen to black
            Some(ClearValue::Color([0.0, 0.0, 0.0, 1.0].into())),
        );

        let depth = graph_builder.create_image(
            window_kind,
            1,
            Format::D32Sfloat,
            Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
        );

        // Create our single `Subpass`, which is the DrawFlat2D pass.
        // We pass the subpass builder a description of our pass for construction
        let pass = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawFlat2DDesc::new().builder())
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        // Finally, add the pass to the graph
        let _present = graph_builder
            .add_node(PresentNode::builder(factory, surface, color).with_dependency(pass));

        graph_builder
    }
}
```

Here we are creating an `ExampleGraph` struct and implementing a `GraphCreator` trait for it.
This trait is responsible for setting up all the details of our rendering pipeline.

> **Note:** This setup code is directly using `rendy` crate to define the rendering.
> You can read about its concepts in the [rendy graph docs][graph].

The important thing to note is that this renders a black background.
It is also ready to draw 2D sprites for us, which we will use in the next chapter.

If you want to use a different background color, you can tweak the RGBA
values inside `ClearValue::Color`. Values range from `0.0` to `1.0`,
so to get that cool green color you can try `[0.00196, 0.23726, 0.21765, 1.0]`.

Now let's pack everything up and run it back in the `main()` function. We have to
expand the existing `GameDataBuilder` with `RenderingSystem` that uses our graph:

```rust,ignore
let game_data = GameDataBuilder::default()
    // The WindowBundle provides all the scaffolding for opening a window
    .with_bundle(WindowBundle::from_config_path(display_config_path))?
    // A Processor system is added to handle loading spritesheets.
    .with(
        Processor::<SpriteSheet>::new(),
        "sprite_sheet_processor",
        &[],
    )
    // The renderer must be executed on the same thread consecutively, so we initialize it as thread_local
    // which will always execute on the main thread.
    .with_thread_local(RenderingSystem::<DefaultBackend, _>::new(
        ExampleGraph::default(),
    ));

let assets_dir = app_root.join("assets/");

let mut game = Application::new(assets_dir, Pong, game_data)?;
game.run();
```

Here we're creating a new `RenderingSystem`, adding the `ExampleGraph` we
created. Additionally we are adding a `Processor::<SpriteSheet>` system,
which will make sure that all `SpriteSheet` assets are being properly loaded.
We will learn more about those in the next chapter.

Success! Now we can compile and run this code with `cargo run` and
get a window. It should look something like this:

![Step one](../images/pong_tutorial/pong_01.png)

[ron]: https://github.com/ron-rs/ron
[simplestate]: https://docs-src.amethyst.rs/stable/amethyst/prelude/trait.SimpleState.html
[state]: https://docs-src.amethyst.rs/stable/amethyst/prelude/trait.State.html
[ap]: https://docs-src.amethyst.rs/stable/amethyst/type.Application.html
[log]: https://docs-src.amethyst.rs/stable/amethyst/struct.Logger.html
[displayconf]: https://docs-src.amethyst.rs/stable/amethyst_renderer/struct.DisplayConfig.html
[graph]: https://github.com/amethyst/rendy/blob/master/docs/graph.md

