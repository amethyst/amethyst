# Creating a Window

Let's start a new project:

`amethyst new pong`

Update the dependencies in the project's `Cargo.toml` so that it contains:

```
[package]
name = "pong"
version = "0.1.0"
authors = []
edition = "2018"

[dependencies.amethyst]
git = "https://github.com/amethyst/amethyst.git"
features = ["vulkan"]
```

In the `src` directory there's a `main.rs` file. Delete everything in that file,
then add these imports:

```rust,edition2018,no_run,noplaypen
//! Pong Tutorial 1

use amethyst::{
    assets::Processor,
    ecs::{ReadExpect, Resources, SystemData},
    prelude::*,
    renderer::{
        pass::DrawFlat2DDesc,
        rendy::{
            factory::Factory,
            graph::{
                render::{RenderGroupDesc, SubpassBuilder},
                GraphBuilder,
            },
            hal::{format::Format, image},
        },
        sprite::SpriteSheet,
        types::DefaultBackend,
        GraphCreator, RenderingSystem,
    },
    utils::application_root_dir,
    window::{ScreenDimensions, Window, WindowBundle},
};
use std::sync::Arc;
```

We'll be learning more about these as we go through this tutorial. The prelude
includes the basic (and most important) types like `Application`, `World`, and
`State`.

Now we create our core game struct:

```rust,edition2018,no_run,noplaypen
pub struct Pong;
```

We'll be implementing the [`SimpleState`][st] trait on this struct, which is
used by Amethyst's state machine to start, stop, and update the game. But for
now we'll just use the default methods provided by `SimpleState`:

```rust,edition2018,no_run,noplaypen
impl SimpleState for Pong {}
```

The `SimpleState` already implements a bunch of stuff for us, like the `update`
and `handle_event` methods that you would have to implement yourself were you
using just a regular `State`. In particular, the default implementation for
`handle_event` returns `Trans::Quit` when a close signal is received
from your operating system, like when you press the close button in your
graphical environment. This allows the application to quit as needed. The
default  implementation for `update` then just returns `Trans::None`, signifying
that nothing is supposed to happen.

Now that we know we can quit, let's add some code to actually get things
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

Inside `main()` we first start the amethyst logger with a default `LoggerConfig`
so we can see errors, warnings and debug messages while the program is running.

```rust,edition2018,no_run,noplaypen
    amethyst::start_logger(Default::default());
```

After the logger is started, we need to create a `DisplayConfig` to store the
configuration for our game's display. We can either define the configuration in
our code, or better yet load it from a file. The latter approach is handier, as
it allows us to change configuration (e.g, the display size) without having to
recompile our game every time.

Starting the project with `amethyst new` should have automatically generated 
`DisplayConfig` data in `resources/display_config.ron`. If you created the
project manually, go ahead and create those now.

In either case, open `display_config.ron` and change its contents to the
following:

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
> learn more about the RON syntax, you can visit the [official repository][ron].

This will set the default window dimensions to 500 x 500, and make the title bar
say "Pong!" instead of the sad, lowercase default of "pong".

In `main()` in `main.rs`, we will load the configuration from the file:

```rust,edition2018,no_run,noplaypen
    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("resources/display_config.ron");
```

Now, let's copy and paste some rendering code so we can keep moving. We'll cover
rendering in more depth later in this tutorial, but for now place the following
functions _below_ the `main()` function:

```rust,edition2018,no_run,noplaypen
// This graph structure is used for creating a proper `RenderGraph` for rendering.
// A renderGraph can be thought of as the stages during a render pass. In our case,
// we are only executing one subpass (DrawFlat2D, or the sprite pass). This graph
// also needs to be rebuilt whenever the window is resized, so the boilerplate code
// for that operation is also here.
#[derive(Default)]
struct ExampleGraph {
    dimensions: Option<ScreenDimensions>,
    surface_format: Option<Format>,
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
        let window = <ReadExpect<'_, Arc<Window>>>::fetch(res);

        // Create a new drawing surface in our window
        let surface = factory.create_surface(&window);
        // cache surface format to speed things up
        let surface_format = *self
            .surface_format
            .get_or_insert_with(|| factory.get_surface_format(&surface));
        let dimensions = self.dimensions.as_ref().unwrap();
        let window_kind = image::Kind::D2(
            dbg!(dimensions.width()) as u32,
            dimensions.height() as u32,
            1,
            1,
        );

        // Begin building our RenderGraph
        let mut graph_builder = GraphBuilder::new();
        let color = graph_builder.create_image(
            window_kind,
            1,
            surface_format,
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
        let sprite = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawFlat2DDesc::new().builder())
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        // Finally, add the pass to the graph
        let _present = graph_builder
            .add_node(PresentNode::builder(factory, surface, color).with_dependency(sprite));

        graph_builder
    }
}
```

The important thing to know right now is that this renders a black background.
If you want a different color you can tweak the RGBA values inside the
`ExampleGraph`'s `builder` method. Values range from 0.0 to 1.0, so to get that cool green color we started with back then, for instance, you can try
`[0.00196, 0.23726, 0.21765, 1.0]`.

Now let's pack everything up and run it back in the `main()` function:

```rust,edition2018,no_run,noplaypen
    let game_data = GameDataBuilder::default()
        // The WindowBundle provides all the scaffolding for opening a window and drawing to it
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

We've discovered Amethyst's root object: [Application][ap]. It binds the OS
event loop, state machines, timers and other core components in a central place.
Here we're creating a new `RenderingSystem`, adding the `ExampleGraph` we
created, along with our config, and building.

Then we call `.run()` on `game` which begins the gameloop. The game will
continue to run until our `SimpleState` returns `Trans::Quit`, or when all states
have been popped off the state machine's stack.

Success! Now we should be able to compile and run this code with `cargo run` and
get a window. It should look something like this:

![Step one](../images/pong_tutorial/pong_01.png)

[ron]: https://github.com/ron-rs/ron
[st]: https://docs-src.amethyst.rs/stable/amethyst/prelude/trait.SimpleState.html
[ap]: https://docs-src.amethyst.rs/stable/amethyst/type.Application.html
[displayconf]: https://docs-src.amethyst.rs/stable/amethyst_renderer/struct.DisplayConfig.html
