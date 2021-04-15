# Drawing the paddles

Now let's do some drawing! But to draw something, we need something to draw. In
Amethyst, those "somethings" are called entities.

Amethyst uses an Entity-Component-System (ECS) framework called **legion**, also
written in Rust.

The term ECS is shorthand for Entity-Component-System. These are the three
core concepts. Each **entity** is associated with some **components**. Those entities
and components are processed by **systems**. This way, you have your data
(components) completely separated from the behavior (systems). An entity just
logically groups components; so a Velocity component can be applied to the
Position component of the same entity.

For more information about the ECS architecture, consult the [legion docs][lg].

## A quick refactor

Before adding more of the Pong logic, we are going to separate the application
initialization code from the Pong code.

1. In the `src` directory, create a new file called `pong.rs` and add the
   following `use` statements. These are needed to make it through this chapter:

   ```rust
    use amethyst::{
        assets::{DefaultLoader, Handle, Loader, ProcessingQueue},
        core::transform::Transform,
        prelude::*,
        renderer::{Camera, SpriteRender, SpriteSheet, Texture},
    };
   ```

1. Move the `Pong` struct and the `impl SimpleState for Pong` block from
   `main.rs` into `pong.rs`.

1. In `main.rs` declare `pong` as a module and import the `Pong` state:

   ```rust
   mod pong;

   use crate::pong::Pong;
   ```

## Get around the World

First, in `pong.rs`, let's add a new method to our `State` implementation: `on_start`.
This method is called when the State starts. We will leave it empty for now.

```rust
# use amethyst::prelude::*;
# struct Pong;
impl SimpleState for Pong {
    fn on_start(&mut self, data: StateData<'_, GameData>) {}
}
```

The `StateData<'_, GameData>` is a structure given to all State methods.
The important part of its content here is its `world` field.

The `World` structure stores all of the game's runtime data -- entities and components.

## Rendering the game using the Camera

The first thing we will need in our game is a `Camera`. This is the component that
will determine what is rendered on screen. It behaves like a real-life
camera: it looks at a specific part of the world and can be moved around at
will.

1. Define the size of the playable area at the top of `pong.rs`.

   ```rust
   pub const ARENA_HEIGHT: f32 = 100.0;
   pub const ARENA_WIDTH: f32 = 100.0;
   ```

   These are public as they will be used in other modules.

1. Create the camera entity.

   In pong, we want the camera to cover the entire arena. Let's do it in a new function `initialize_camera`:

    ```rust
    # use amethyst::{
    #     assets::{DefaultLoader, Handle, Loader, ProcessingQueue},
    #     core::transform::Transform,
    #     prelude::*,
    #     renderer::{Camera, SpriteRender, SpriteSheet, Texture},
    # };

    # const ARENA_HEIGHT: f32 = 100.0;
    # const ARENA_WIDTH: f32 = 100.0;
    fn initialize_camera(world: &mut World) {
        // Setup camera in a way that our screen covers whole arena and (0, 0) is in the bottom left.
        let mut transform = Transform::default();
        transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);

        world.push((Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT), transform));
    }
    ```

   This creates an entity that will carry our camera, with an orthographic
   projection of the size of our arena. We also attach a `Transform` component,
   representing its position in the world.

   The `Camera::standard_2d` function creates a default 2D camera that is
   pointed along the world's **Z** axis. The area in front of the camera has a
   horizontal **X** axis, and a vertical **Y** axis. The **X** axis increases
   moving to the right, and the **Y** axis increases moving up. The camera's
   position is the center of the viewable area. We position the camera with
   `set_translation_xyz` to the middle of our game arena so that `(0, 0)` is
   the bottom left of the viewable area, and `(ARENA_WIDTH, ARENA_HEIGHT)` is
   the top right.

   Notice that we also shifted the camera `1.0` along the **Z** axis. This is
   to make sure that the camera is able to see the sprites that sit on the
   **XY** plane where **Z** is 0.0:

   ![Camera Z shift](../images/pong_tutorial/camera.png)

   > **Note:** Orthographic projections are a type of 3D visualization on 2D screens
   > that keeps the size ratio of the 2D images displayed intact. They are very
   > useful in games without actual 3D, like our pong example. Perspective projections
   > are another way of displaying graphics, more useful in 3D scenes.

2. To finish setting up the camera, we need to call `initialize_camera` from the
   Pong state's `on_start` method:

   ```rust
    # use amethyst::{
    #     assets::{DefaultLoader, Handle, Loader, ProcessingQueue},
    #     core::transform::Transform,
    #     prelude::*,
    #     renderer::{Camera, SpriteRender, SpriteSheet, Texture},
    # };

    # const ARENA_HEIGHT: f32 = 100.0;
    # const ARENA_WIDTH: f32 = 100.0;
    pub struct Pong;

    impl SimpleState for Pong {
        fn on_start(&mut self, data: StateData<'_, GameData>) {
            let world = data.world;

            initialize_camera(world);
        }
    }
   ```

Now that our camera is set up, it's time to add the paddles.

## Our first Component

Now, we will create the `Paddle` component, all in `pong.rs`.

1. Define constants for the paddle width and height.

   ```rust
   pub const PADDLE_HEIGHT: f32 = 16.0;
   pub const PADDLE_WIDTH: f32 = 4.0;
   ```

1. Define the `Side` enum and `Paddle` struct:

   ```rust
   # pub const PADDLE_HEIGHT: f32 = 16.0;
   # pub const PADDLE_WIDTH: f32 = 4.0;
   # 
   #[derive(PartialEq, Eq)]
   pub enum Side {
       Left,
       Right,
   }

   pub struct Paddle {
       pub side: Side,
       pub width: f32,
       pub height: f32,
   }

   impl Paddle {
       fn new(side: Side) -> Paddle {
           Paddle {
               side,
               width: PADDLE_WIDTH,
               height: PADDLE_HEIGHT,
           }
       }
   }
   ```

   *"But that just looks like a regular struct!"* you might say.

   Legion will take care of creating the archetypes for optimal storage automatically based on the combination of components given to an entity.

## initialize some entities

Now that we have a `Paddle` component, let's define some paddle entities that
include that component and add them to our `World`.

First let's look at our imports:

```rust
use amethyst::core::transform::Transform;
```

`Transform` is an Amethyst ECS component which carries
position and orientation information. It is relative
to a parent, if one exists.

Okay, let's make some entities! We'll define an `initialize_paddles` function
which will create left and right paddle entities and attach a `Transform`
component to each to position them in our world. As we defined earlier,
our canvas is from `0.0` to `ARENA_WIDTH` in the horizontal dimension and
from `0.0` to `ARENA_HEIGHT` in the vertical dimension.
Keep in mind that the anchor point of our entities will be in the middle of the
image we will want to render on top of them. This is a good rule to follow in
general, as it makes operations like rotation easier.

```rust
/// initializes one paddle on the left, and one paddle on the right.
fn initialize_paddles(world: &mut World) {
    let mut left_transform = Transform::default();
    let mut right_transform = Transform::default();

    // Correctly position the paddles.
    let y = ARENA_HEIGHT / 2.0;
    left_transform.set_translation_xyz(PADDLE_WIDTH * 0.5, y, 0.0);
    right_transform.set_translation_xyz(ARENA_WIDTH - PADDLE_WIDTH * 0.5, y, 0.0);

    // Create left plank entity.
    world.push((Paddle::new(Side::Left), left_transform));

    // Create right plank entity.
    world.push((Paddle::new(Side::Right), right_transform));
}
```

This is all the information Amethyst needs to track and move the paddles in our
virtual world, but we'll need to do some more work to actually *draw* them.

As a sanity check, let's make sure the code for initializing the paddles
compiles. Update the `on_start` method to the following:

```rust
# use amethyst::ecs::World;
# use amethyst::prelude::*;
# fn initialize_paddles(world: &mut World) {}
# fn initialize_camera(world: &mut World) {}
# struct MyState;
# impl SimpleState for MyState {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let world = data.world;

        initialize_paddles(world);
        initialize_camera(world);
    }
# }
```

## Bundles

Amethyst has a lot of internal systems it uses to keep things running we need
to bring into the context of the `World`. For simplicity, these have been
grouped into "Bundles" which include related systems and resources. We can
add these to our Application's `GameData` using the `add_bundle` method,
similarly to how you would register a system. We already have `RenderBundle` in place,
registering another one will look similar. You have to first import
`TransformBundle`, then register it as follows:

```rust
use amethyst::{
#   assets::LoaderBundle,
#   // Add TransformBundle to the existing use statement
#   core::transform::TransformBundle,
#   prelude::*,
#   renderer::{
#       plugins::{RenderFlat2D, RenderToWindow},
#       rendy::hal::command::ClearColor,
#       types::DefaultBackend,
#       RenderingBundle,
#   },
#   utils::application_root_dir,
};
fn main() -> amethyst::Result<()> {
    # amethyst::start_logger(Default::default());
    # 
    # let app_root = application_root_dir()?;
    # let display_config_path = app_root.join("config/display.ron");
    # 
    # let assets_dir = app_root.join("assets/");

    // --snip--
    let mut dispatcher = DispatcherBuilder::default();
    dispatcher
        // Add the transform bundle which handles tracking entity positions        
        .add_bundle(TransformBundle)
        // --snip--

    # let game = Application::new(assets_dir, Pong, dispatcher)?;
    # game.run();
    # Ok(())
}
```

When you run the game, you should see the familiar black screen.

## Drawing

This section will finally allow us to see something.

The first thing we will have to do is load the sprite sheet we will use for all
our graphics in the game. Create a `texture` folder in the `assets` directory of the project.
This will contain the [spritesheet texture][ss] `pong_spritesheet.png`, which we
need to render the elements of the game.  We will perform the loading in a new
function in `pong.rs` called `load_sprite_sheet`.

First, let's declare the function and load the sprite sheet's image data.

```rust
# use amethyst::{
#     assets::{DefaultLoader, Handle, Loader, ProcessingQueue},
#     core::transform::Transform,
#     prelude::*,
#     renderer::{Camera, SpriteRender, SpriteSheet, Texture},
# };
# 
fn load_sprite_sheet(resources: &mut Resources) -> Handle<SpriteSheet> {
    // Load the sprite sheet necessary to render the graphics.
    // `texture` is a cloneable reference to the pixel data.
    let texture: Handle<Texture> = {
        let loader = resources.get::<DefaultLoader>().unwrap();
        loader.load("texture/pong_spritesheet.png")
    };

    // ...
#   unimplemented!()
}
```

The `Loader` is an asset loader which is defined as a resource (not an
`Entity`, `Component`, or `System`, but still a part of our ECS `World`). It was
created when we built our Application in `main.rs`, and it can read assets like
.obj files, but also it can `load` a .png as a `Texture` as in our use case.

> Resources in Legion are a type of data which can be shared between systems,
> while being independent of entities, in contrast to components, which are
> attached to specific entities.

In order to manage assets while remaining fast, Amethyst does not give us direct access to the assets
we load. If it did otherwise, we would have to wait for the texture to be fully loaded to do all the
other things we have to prepare, which would be a waste of time!

Instead, the `load` function will return a `Handle<Texture>`. This handle "points" to the place where the asset will be loaded. In Rust terms, it is equivalent to a reference-counted option. It is extremely useful, especially as cloning the handle does not clone the asset in memory, so many things can use the same asset at once.

Alongside our sprite sheet texture, we need a file describing where the sprites
are on the sheet. Let's create, right next to it, a file called
`pong_spritesheet.ron`. It will contain the following sprite sheet definition:

```text
/*!
    @import /amethyst_rendy/src/sprite/mod.rs#Sprites
    Sprites
*/
{
    "04c60333-c790-4586-aa76-086b19167a04":
    List((
        texture_width: 8,
        texture_height: 16,
        sprites: [
            (
                x: 0,
                y: 0,
                width: 4,
                height: 16,
            ),
            (
                x: 4,
                y: 0,
                width: 4,
                height: 4,
            ),
        ],
    ))
}
```

> **Note:** Make sure to pay attention to the kind of parentheses in the ron file.
> Especially, if you are used to writing JSON or similar format files, you might
> be tempted to use curly braces there; that will however lead to very
> hard-to-debug errors, especially since amethyst will not warn you about that
> when compiling.

Finally, we load the file containing the position of each sprite on the sheet.

```rust
# use amethyst::{
#     assets::{DefaultLoader, Handle, Loader, ProcessingQueue},
#     core::transform::Transform,
#     prelude::*,
#     renderer::{Camera, SpriteRender, SpriteSheet, Texture},
# };
# 
fn load_sprite_sheet(resources: &mut Resources) -> Handle<SpriteSheet> {
#   // Load the sprite sheet necessary to render the graphics.
#   // `texture` is a cloneable reference to the pixel data.
#   let texture: Handle<Texture> = {
#       let loader = resources.get::<DefaultLoader>().unwrap();
#       loader.load("texture/pong_spritesheet.png")
#   };
# 
    // --snip--

    let loader = resources.get::<DefaultLoader>();
    let sprites = loader.load("texture/pong_spritesheet.ron");
    loader.load_from_data(
        SpriteSheet { texture, sprites },
        (),
        &resources.get::<ProcessingQueue<SpriteSheet>>().unwrap(),
    )
# }
```

This is where we have to use the texture handle. First, the `Loader` takes the
file containing the sprites' positions and returns a handle to a `SpriteList`. 

Then, the `Loader` takes a `SpriteSheet` struct, which contains both the texture handle
and the sprite handle. It is this struct that we will be using to actually draw stuff on the screen.

Please note that the order of sprites declared in the sprite sheet file
is also significant, as sprites are referenced by the index in
the vector. If you're wondering about the ball sprite, it does exist on the
image, but we will get to it in a later part of the tutorial.

So far, so good. We have a sprite sheet loaded, now we need to link the sprites
to the paddles. We update the `initialize_paddles` function by changing its
signature to:

```rust
# use amethyst::ecs::World;
# use amethyst::{assets::Handle, renderer::sprite::SpriteSheet};
fn initialize_paddles(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>)
# {
# }
```

Inside `initialize_paddles`, we construct a `SpriteRender` for a paddle. We
only need one here, since the only difference between the two paddles is that
the right one is flipped horizontally.

```rust
# use amethyst::ecs::World;
# use amethyst::{
#   assets::Handle,
#   renderer::{SpriteRender, SpriteSheet},
# };
# fn initialize_paddles(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
    // Assign the sprites for the paddles
    let sprite_render = SpriteRender::new(sprite_sheet_handle, 0); // paddle is the first sprite in the sprite_sheet
# }
```

`SpriteRender` is the `Component` that indicates which sprite of which sprite
sheet should be drawn for a particular entity. Since the paddle is the first
sprite in the sprite sheet, we use `0` for the `sprite_number`.

Next we simply add the components to the paddle entities:

```rust
# use amethyst::assets::Handle;
# use amethyst::ecs::World;
# use amethyst::prelude::*;
# use amethyst::renderer::sprite::{SpriteRender, SpriteSheet};
# fn initialize_paddles(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
#   let sprite_render = SpriteRender::new(sprite_sheet_handle, 0); // paddle is the first sprite in the sprite_sheet
                                                                   // Create a left plank entity.
    world.push((sprite_render.clone() /* ... other components */,));

    // Create right plank entity.
    world.push((sprite_render /* ... other components */,));

# }
```

We're nearly there, we have to wire up the sprite to the paddles. We put it
all together in the `on_start()` method:

```rust
# use amethyst::assets::Handle;
# use amethyst::ecs::World;
# use amethyst::prelude::*;
# use amethyst::renderer::{sprite::SpriteSheet, Texture};
# struct Paddle;
# fn initialize_paddles(world: &mut World, spritesheet: Handle<SpriteSheet>) {}
# fn initialize_camera(world: &mut World) {}
# fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
#   unimplemented!()
# }
# struct MyState;
# impl SimpleState for MyState {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let world = data.world;

        // Load the spritesheet necessary to render the graphics.
        let sprite_sheet_handle = load_sprite_sheet(world);

        world.register::<Paddle>();

        initialize_paddles(world, sprite_sheet_handle);
        initialize_camera(world);
    }
# }
```

And we're done. Let's run our game and have fun!

If all is well, we should get something that looks like this:

![Step two](../images/pong_tutorial/pong_02.png)

Hooray!

In the next chapter, we'll explore the "S" in ECS and actually get these paddles
moving!

[lg]: https://docs.rs/legion/0.4.0/legion/#getting-started
[ss]: ../images/pong_tutorial/pong_spritesheet.png
