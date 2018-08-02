# Drawing the paddles

Now let's do some drawing! But to draw something, we need something to draw. In
Amethyst, those "somethings" are called Entities, which are described by
Components.

Amethyst uses Specs for its ECS (Entity-component system), which is a parallel
Entity-component system written in Rust. You can learn more about Specs in the
[The Specs Book][sb]. Here's a basic explanation of ECS from there:

> The term ECS is a shorthand for Entity-component system. These are the three
> core concepts. Each entity is associated with some components. Those entities
> and components are processed by systems. This way, you have your data
> (components) completely separated from the behaviour (systems). An entity just
> logically groups components; so a Velocity component can be applied to the
> Position component of the same entity.

I recommend at least skimming the rest of The Specs Book to get a good intuition
of how Amethyst works, especially if you're new to ECS.

## A quick refactor

Let's create a new file called `pong.rs` to hold our core game logic. We can
move the `Pong` struct over here, and the `impl State for Pong` block as well.
Then, in `main.rs` declare a module:

```rust,ignore
mod pong;
```

And in the `run()` function add:

```rust,ignore
use pong::Pong;
```

Now you can just delete various `main.rs` use statements until the Rust compiler
stops complaining about unused imports. In `pong.rs` we'll need these use
statements to make it through this chapter:

```rust,ignore
use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::cgmath::{Vector3, Matrix4};
use amethyst::core::transform::{GlobalTransform, Transform};
use amethyst::ecs::prelude::{Component, DenseVecStorage};
use amethyst::input::{is_close_requested, is_key_down};
use amethyst::prelude::*;
use amethyst::renderer::{
    Camera, Event, PngFormat, Projection, Sprite, Texture, TextureHandle,
    VirtualKeyCode, WithSpriteRender,
};
```

## Get around the World

First, in `pong.rs`, let's add a new method to our State implementation: `on_start`.
This method is called, as you probably guessed, when the State starts.
We will leave it empty for now, but it will become useful later down the line.

```rust,ignore
fn on_start(&mut self, data: StateData<GameData>) {

}
```

The `StateData<GameData>` is a structure given to all State methods. The important
part of its content here is its `world` field.

The `World` structure gets passed around everywhere. It carries with it all the
elements of the runtime of our game: entities, components and systems.
Remember when we added bundles in our `main.rs`, they were in fact adding
all the systems they were holding inside the `World` before we actually
ran the game.

## Look at your game through the Camera

The first thing we will need in our game is a Camera. This is the component
that will determine what is rendered on screen. It behaves just like a
real life camera: it records a specific part of the world and can be
moved around at will.

First, let's define some constants:

```rust,ignore
const ARENA_HEIGHT: f32 = 100.0;
const ARENA_WIDTH: f32 = 100.0;
```

These constants will determine the size of our arena.
So, as we're making a pong game, we want to create a camera that will cover
the entire arena. Let's do it!

```rust,ignore
fn initialise_camera(world: &mut World) {
    world.create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0,
            ARENA_WIDTH,
            ARENA_HEIGHT,
            0.0,
        )))
        .with(GlobalTransform(
            Matrix4::from_translation(Vector3::new(0.0, 0.0, 1.0)).into()
        ))
        .build();
}
```

We create an entity that will carry our camera, with an orthographic projection
of the size of our arena (as we want it to cover it all). Ignore the
`GlobalTransform` for now, we'll deal with it in more details later on.
Note that as the origin of our camera is in the bottom left corner, we set
`ARENA_HEIGHT` as the top and `0.0` as the bottom.

> Orthographic projections are a type of 3D visualization on 2D screens
> that keeps the size ratio of the 2D images displayed intact. They are very
> useful in games without actual 3D, like our pong example. Perspective projections
> are another way of displaying graphics, more useful in 3D scenes.

## Our first Component

In `pong.rs` let's create our first `Component`, a definition of a paddle.

```rust,ignore
#[derive(PartialEq, Eq)]
enum Side {
    Left,
    Right,
}

struct Paddle {
    pub side: Side,
    pub width: f32,
    pub height: f32,
}

impl Paddle {
    fn new(side: Side) -> Paddle {
        Paddle {
            side: side,
            width: 1.0,
            height: 1.0,
        }
    }
}
```

"But that just looks like a regular struct!" you might say. And you're right,
here's the special sauce:

```rust,ignore
impl Component for Paddle {
    type Storage = DenseVecStorage<Self>;
}
```

By implementing `Component` for our `Paddle` struct, and defining the way we'd
like that `Component` data stored, we can now add the `Paddle` component to
entities in our game. For more on storage types, check out the
[Specs documentation][sb-storage].

## Initialise some entities

Now that we have a Paddle component, let's define some paddle entities that
include that component and add them to our `World`.

First let's look at our math imports:

```rust,ignore
use amethyst::core::cgmath::{Vector3, Matrix4};
use amethyst::core::transform::{GlobalTransform, Transform};
```

Amethyst uses the [cgmath crate][cg] under the hood and exposes it for our use.
Today we just grabbed the `Vector3` type, which is a very good math thing to have.
(we also grabbed `Matrix4` for the `GlobalTransform` earlier, but we won't use it here)

`Transform` and `GlobalTransform` are Amethyst ECS components which carry
position and orientation information. `Transform` is relative
to a parent if one exists, while `GlobalTransform` is, well, global.

Let's also define some constants for convenience:

```rust,ignore
const PADDLE_HEIGHT: f32 = 16.0;
const PADDLE_WIDTH: f32 = 4.0;
```

Okay, let's make some entities! We'll define an `initialise_paddles` function
which will create left and right paddle entities and attach a `Transform`
component to each to position them in our world. As we defined earlier,
our canvas is from `0.0` to `ARENA_WIDTH` in the horizontal dimension and
from `0.0` to `ARENA_HEIGHT` in the vertical dimension.
Keep in mind that the anchor point of our entities will be in the middle of the
image we will want to render on top of them. This is a good rule to follow in
general as it makes operations like rotation easier.

```rust,ignore
/// Initialises one paddle on the left, and one paddle on the right.
fn initialise_paddles(world: &mut World) {
    let mut left_transform = Transform::default();
    let mut right_transform = Transform::default();

    // Correctly position the paddles.
    let y = ARENA_HEIGHT / 2.0;
    left_transform.translation = Vector3::new(PADDLE_WIDTH * 0.5, y, 0.0);
    right_transform.translation = Vector3::new(ARENA_WIDTH - PADDLE_WIDTH * 0.5, y, 0.0);

    // Create a left plank entity.
    world
        .create_entity()
        .with(Paddle::new(Side::Left))
        .with(GlobalTransform::default())
        .with(left_transform)
        .build();

    // Create right plank entity.
    world
        .create_entity()
        .with(Paddle::new(Side::Right))
        .with(GlobalTransform::default())
        .with(right_transform)
        .build();
}
```

This is all the information Amethyst needs to track and move the paddles in our
virtual world, but we'll need to do some more work to actually *draw* them.

## Drawing

The first thing we will have to do is load the sprite sheet we will use for all our
graphics in the game. Here, it is located in `texture/pong_spritesheet.png`.
We will perform the loading in the `on_start` method.

```rust,ignore
let world = data.world;

// Load the spritesheet necessary to render the graphics.
let spritesheet = {
    let loader = world.read_resource::<Loader>();
    let texture_storage = world.read_resource::<AssetStorage<Texture>>();
    loader.load(
        "texture/pong_spritesheet.png",
        PngFormat,
        Default::default(),
        (),
        &texture_storage,
    )
};
```

The `Loader` is an asset loader which is defined as a `resource` (not a `Entity`
, `Component`, or `System`, but still a part of our ECS `world`). It was created
when we built our Application in `main.rs`, and it can read assets like .obj
files, but also it can `load` a .png as a `Texture` as in our use case.

> Resources in Specs are a type of data which can be shared between systems,
> while being independent from entities, in contrast to components, which are
> attached to specific entities. We'll explore this more later on.

The `AssetStorage<Texture>` is also a `resource`, this is where the loader will
put the `Texture` it will load from our sprite sheet. In order to manage them
while remaining fast, Amethyst does not give us direct access to the assets we load.
If it did otherwise, we would have to wait for the texture to be fully loaded to do all the
other things we have to prepare, which would be a waste of time!
Instead, the `load` function will return a `Handle<Texture>` (also known as `TextureHandle`).
This handle "points" to the place where the asset will be loaded. In Rust terms, it is
equivalent to a reference-counted option. It is extremely useful, especially as cloning
the handle does not clone the asset in memory, so many things can use the same asset at once.

Now that we have a handle to our sprite sheet's texture, we can communicate it to our
`initialise_paddle` function by changing its signature to:

```rust,ignore
fn initialise_paddles(world: &mut World, spritesheet: TextureHandle)
```

We now need to define what part of the spritesheet we want to render.
To do that, we need to create a `Sprite`, which is a fancy name to call a rectangle on
the sprite sheet, before the creation of the entities. This is dead simple:

```rust,ignore
// Build the sprite for the paddles.
let sprite = Sprite {
    left: 0.0,
    right: PADDLE_WIDTH,
    top: 0.0,
    bottom: PADDLE_HEIGHT,
};
```

Here, we take the rectangle from `(0.0 ; 0.0)` to `(PADDLE_WIDTH ; PADDLE_HEIGHT)`
on the sprite sheet to be displayed.

Then, using the `WithSpriteRender` trait, we can easily modify our
entity creation code to have the entities display the sprite.

```rust,ignore
const SPRITESHEET_SIZE: (f32, f32) = (8.0, 16.0);

// Create a left plank entity.
world
    .create_entity()
    .with_sprite(&sprite, spritesheet.clone(), SPRITESHEET_SIZE)
    .expect("Failed to add sprite render on left paddle")
    .with(Paddle::new(Side::Left))
    .with(GlobalTransform::default())
    .with(left_transform)
    .build();

// Create right plank entity.
world
    .create_entity()
    .with_sprite(&sprite, spritesheet, SPRITESHEET_SIZE)
    .expect("Failed to add sprite render on right paddle")
    .with(Paddle::new(Side::Right))
    .with(GlobalTransform::default())
    .with(right_transform)
    .build();
```

Please note that we need to manually specify the size of our sprite sheet.
This is because if we happened to add our sprite to the entity while the
sprite sheet is not loaded yet, there would be no way for the renderer to
get the size of the texture it needs to do its magic.

> Behind the scene, the `with_sprite` method adds `Mesh` and `Material`
> components to your entity. It is a utility function to spare you from
> the rendering details, but Amethyst can expose all the precision of
> the rendering process if you need it. Many utility functions like this
> exist in Amethyst to make prototyping easier.

> Here, we are using the `with_sprite` utility twice for the same sprite.
> Keep in mind however that another syntax exists in the `SpriteRenderData` struct
> when we need multiple entities to display the exact same sprite, leading
> to improved performance. Here, however, it is negligible.

Now let's add our initialise functions to the `on_start` function in `impl State
for Pong`. It now looks like this:

```rust,ignore
fn on_start(&mut self, data: StateData<GameData>) {
    let world = data.world;

    // Load the spritesheet necessary to render the graphics.
    let spritesheet = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "texture/pong_spritesheet.png",
            PngFormat,
            Default::default(),
            (),
            &texture_storage,
        )
    };

    initialise_paddles(world, spritesheet);
    initialise_camera(world);
}
```

Okay! We've defined our `Paddle` component, and created two entities which have
a `Paddle` component, a `GlobalTransform` component and a sprite. When our game
starts, we'll add the left and right paddles to the world, along with a camera.

Before we continue, one last note.
Components do not have to be registered manually to be used, however you need to have
something that uses them to have them be registered automatically.
As nothing uses our `Paddle` component yet, we will register it manually before we initialise
our paddles in the `on_start` method by calling:

```rust,ignore
world.register::<Paddle>();
```

And we're done.
Let's run our game and have fun for days!

```
thread 'main' panicked at 'Tried to fetch a resource, but the resource does not exist.
Try adding the resource by inserting it manually or using the `setup` method.
```

Ah, oops. We forgot something.

Amethyst has a lot of internal systems it uses to keep things running we need to bring
into the context of the `World`. For simplicity, these have been wrapped up into "Bundles"
which include related systems and resources. We can add these to our Application using the
`with_bundle` method, and in fact we already have one of these in `main.rs`: the `RenderBundle`.

As it turns out, the system we're missing is `TransformSystem`, and we can add it with the 
`TransformBundle`.

```rust,ignore
let mut game = Application::build("./", Pong)?
    .with_bundle(TransformBundle::new())? // Add this bundle
    .with_bundle(RenderBundle::new(pipe, Some(config)))?
    .build()?;
```

Also we'll need to import that structure:

```rust,ignore
use amethyst::core::transform::TransformBundle;
```

Now when we run the game we should get something that looks like this:

![Step two](../images/pong_tutorial/pong_02.png)

In the next chapter we'll explore the "S" in ECS and actually get these paddles
moving!

[sb]: https://slide-rs.github.io/specs/
[sb-storage]: https://slide-rs.github.io/specs/05_storages.html#densevecstorage
[cg]: https://docs.rs/cgmath/0.15.0/cgmath/
[2d]: https://www.amethyst.rs/doc/develop/doc/amethyst_renderer/struct.Camera.html#method.standard_2d
