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

And in main.rs, at the top of the file, add this import:

```rust,ignore
use pong::Pong;
```

Now you can just delete various `main.rs` use statements until the Rust compiler
stops complaining about unused imports. In the `pong.rs` file we'll need these use
statements to make it through this chapter:

```rust,no_run,noplaypen
# extern crate amethyst;
use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::cgmath::{Vector3, Matrix4};
use amethyst::core::transform::{GlobalTransform, Transform};
use amethyst::ecs::prelude::{Component, DenseVecStorage};
use amethyst::prelude::*;
use amethyst::renderer::{
    Camera, MaterialTextureSet, PngFormat, Projection, Sprite,
    SpriteRender, SpriteSheet, SpriteSheetHandle, Texture, TextureCoordinates,
};
```

## Get around the World

First, in `pong.rs`, let's add a new method to our State implementation: `on_start`.
This method is called, as you probably guessed, when the State starts.
We will leave it empty for now, but it will become useful later down the line.

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# struct MyState;
# impl<'a, 'b> SimpleState<'a,'b> for MyState {
fn on_start(&mut self, data: StateData<GameData>) {

}
# }
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

First, let's define some constants. We will make them public for use in other
modules later:

```rust,no_run,noplaypen
pub const ARENA_HEIGHT: f32 = 100.0;
pub const ARENA_WIDTH: f32 = 100.0;
```

These constants will determine the size of our arena.
So, as we're making a pong game, we want to create a camera that will cover
the entire arena. Let's do it!

```rust,no_run,noplaypen
# extern crate amethyst;
# const ARENA_HEIGHT: f32 = 100.0;
# const ARENA_WIDTH: f32 = 100.0;
# use amethyst::prelude::*;
# use amethyst::ecs::World;
# use amethyst::renderer::{Camera, Projection};
# use amethyst::core::cgmath::{Vector3, Matrix4};
# use amethyst::core::{Transform, GlobalTransform};
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

To finish setting up the camera, let's call it in our State's `on_start` method:

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# use amethyst::ecs::World;
# fn initialise_camera(world: &mut World) { }
# struct MyState;
# impl<'a, 'b> SimpleState<'a,'b> for MyState {
fn on_start(&mut self, data: StateData<GameData>) {
    let world = data.world;

    initialise_camera(world);
}
# }
```

If you run the game now, you will see... a blank window. Unfortunately this will
be the case until we get to the end of this part of the tutorial, but it gets
much better from then on, we promise!

## Our first Component

In `pong.rs` let's create our first `Component`, a definition of a paddle. We
will make `Side` and `Paddle` public for use in other modules later.

```rust,no_run,noplaypen
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
            side: side,
            width: 1.0,
            height: 1.0,
        }
    }
}
```

"But that just looks like a regular struct!" you might say. And you're right,
here's the special sauce:

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::{Component, DenseVecStorage};
# struct Paddle;
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

```rust,no_run,noplaypen
# extern crate amethyst;
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

```rust,no_run,noplaypen
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

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# use amethyst::core::{Transform, GlobalTransform};
# use amethyst::core::cgmath::Vector3;
# use amethyst::ecs::World;
# enum Side {
#   Left,
#   Right,
# }
# struct Paddle;
# impl amethyst::ecs::Component for Paddle {
#   type Storage = amethyst::ecs::VecStorage<Paddle>;    
# }
# impl Paddle {
#   fn new(side: Side) -> Paddle { Paddle }
# }
# const PADDLE_HEIGHT: f32 = 16.0;
# const PADDLE_WIDTH: f32 = 4.0;
# const ARENA_HEIGHT: f32 = 100.0;
# const ARENA_WIDTH: f32 = 100.0;
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

As a sanity check, let's make sure the code for initialising the paddles
compiles. Update the `on_start` method to the following:

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# use amethyst::ecs::World;
# fn initialise_paddles(world: &mut World) { }
# fn initialise_camera(world: &mut World) { }
# struct MyState;
# impl<'a, 'b> SimpleState<'a,'b> for MyState {
fn on_start(&mut self, data: StateData<GameData>) {
    let world = data.world;

    initialise_paddles(world);
    initialise_camera(world);
}
# }
```

Let's run our blank screen game!

```text,ignore
thread 'main' panicked at 'Tried to fetch a resource, but the resource does not exist.
Try adding the resource by inserting it manually or using the `setup` method.
```

Uh oh, what's wrong? Sadly the message is pretty difficult to decipher.

If you are using a `nightly` compiler and enable the `nightly` feature of
Amethyst, you will receive a more informative error message:

```text,ignore
thread 'main' panicked at 'Tried to fetch a resource of type "amethyst::specs::storage::MaskedStorage<pong::Paddle>", but the resource does not exist.
Try adding the resource by inserting it manually or using the `setup` method.'
```

For a `Component` to be used, there must be a `Storage<ComponentType>` resource
set up in the `World`. The error message above means we have registered the
`Paddle` component on an entity, but have not set up the `Storage`. We can fix
this by adding the following line before `initialise_paddles(world)` in the
`on_start` method:

```rust,no_run,noplaypen
# extern crate amethyst;
# struct Paddle;
# impl amethyst::ecs::Component for Paddle {
#   type Storage = amethyst::ecs::VecStorage<Paddle>;    
# }
# fn register() {
#   let mut world = amethyst::ecs::World::new();
world.register::<Paddle>();
# }
```

This is rather inconvenient &mdash; to need to manually register each component
before it can be used. There *must* be a better way. **Hint:** there is.

When we add systems to our application, any component that a `System` uses is
automatically registered. However, as we haven't got any `System`s we have to
live with registering the `Paddle` component manually.

Let's run the game again.

```text,ignore
thread 'main' panicked at 'Tried to fetch a resource, but the resource does not exist.
Try adding the resource by inserting it manually or using the `setup` method.
```

Ah, oops. We forgot something. Turning on the `nightly` feature, we get:

```text_ignore
thread 'main' panicked at 'Tried to fetch a resource of type "specs::storage::MaskedStorage<transform::components::local_transform::Transform>", but the resource does not exist.
Try adding the resource by inserting it manually or using the `setup` method.'
```

This is the same kind of error as before; this time the `Component` is a
`Transform`, which is used and hence registered by the `TransformSystem`.

Amethyst has a lot of internal systems it uses to keep things running we need
to bring into the context of the `World`. For simplicity, these have been
wrapped up into "Bundles" which include related systems and resources. We can
add these to our Application's `GameData` using the `with_bundle` method. We
already have one of these in `main.rs`: the `RenderBundle`. We can just follow
the pattern and add the `TransformBundle`.

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# use amethyst::core::transform::TransformBundle;
# use amethyst::renderer::{DisplayConfig, DrawSprite, Event, Pipeline,
#                        RenderBundle, Stage, VirtualKeyCode};
# fn main() -> amethyst::Result<()> {
# let path = "./resources/display_config.ron";
# let config = DisplayConfig::load(&path);
# let pipe = Pipeline::build().with_stage(Stage::with_backbuffer()
#       .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
#       .with_pass(DrawSprite::new()),
# );
# struct Pong;
# impl<'a, 'b> SimpleState<'a,'b> for Pong { }
let game_data = GameDataBuilder::default()
    .with_bundle(RenderBundle::new(pipe, Some(config)).with_sprite_sheet_processor())?
    .with_bundle(TransformBundle::new())?;
# Ok(())
# }
```

Also we'll need to import it:

```rust,no_run,noplaypen
# extern crate amethyst;
use amethyst::core::transform::TransformBundle;
```

This time, when you run the game you should see the familiar black screen.
Hooray!

## Drawing

This section will finally allow us to see something.

The first thing we will have to do is load the sprite sheet we will use for all
our graphics in the game. Here, it is located in `texture/pong_spritesheet.png`.
We will perform the loading in a new function called `load_sprite_sheet`. There
is quite a lot to do to load a sprite sheet, so we will construct the
`load_sprite_sheet` function in pieces.

First, let's declare an additional constant for the spritesheet. This will be
used when defining how the sprites are laid out on the sprite sheet.

```rust,no_run,noplaypen
const SPRITESHEET_SIZE: (f32, f32) = (8.0, 16.0);
```

Next, we declare the function and load the image.

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# use amethyst::assets::{Loader, AssetStorage};
# use amethyst::renderer::{Texture, PngFormat, TextureHandle, SpriteSheetHandle};
# use amethyst::ecs::World;
fn load_sprite_sheet(world: &mut World) -> SpriteSheetHandle {
    // Load the sprite sheet necessary to render the graphics.
    // The texture is the pixel data
    // `texture_handle` is a cloneable reference to the texture
    let texture_handle = {
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

    //...
#   unimplemented!()
}
```

The `Loader` is an asset loader which is defined as a `resource` (not an
`Entity`, `Component`, or `System`, but still a part of our ECS `world`). It was
created when we built our Application in `main.rs`, and it can read assets like
.obj files, but also it can `load` a .png as a `Texture` as in our use case.

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

Heading back to the code, we need to add this snippet after loading the texture.

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# use amethyst::assets::{Loader, AssetStorage};
# use amethyst::renderer::{Texture, PngFormat, TextureHandle, MaterialTextureSet, SpriteSheetHandle};
# use amethyst::ecs::World;
# fn load_sprite_sheet(world: &mut World) {
#   let texture_handle = {
#       let loader = world.read_resource::<Loader>();
#       let texture_storage = world.read_resource::<AssetStorage<Texture>>();
#       loader.load(
#           "texture/pong_spritesheet.png",
#           PngFormat,
#           Default::default(),
#           (),
#           &texture_storage,
#       )
#   };
// `texture_id` is a application defined ID given to the texture to store in
// the `World`. This is needed to link the texture to the sprite_sheet.
let texture_id = 0;
let mut material_texture_set = world.write_resource::<MaterialTextureSet>();
material_texture_set.insert(texture_id, texture_handle);
# }
```

The `MaterialTextureSet` is yet another `resource`, which is a bi-directional
map between an application defined texture ID and the handle of the loaded
texture. As you will see in a moment, `SpriteSheet`s are linked to textures
through this ID. Since we only have one sprite sheet, we can just use `0` as the
ID.

We now need to define what part of the texture we want to render. To do that, we
need to create a `Sprite`, which is a fancy name to call a rectangle on the
sprite sheet. Behold, texture coordinates!

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# use amethyst::assets::{Loader, AssetStorage};
# use amethyst::renderer::{Texture, PngFormat, TextureHandle, MaterialTextureSet,
                           TextureCoordinates, Sprite, SpriteSheet, SpriteSheetHandle};
# use amethyst::ecs::World;
# const PADDLE_HEIGHT: f32 = 16.0;
# const PADDLE_WIDTH: f32 = 4.0;
# const SPRITESHEET_SIZE: (f32, f32) = (8.0, 16.0);
# fn load_sprite_sheet(world: &mut World) -> SpriteSheetHandle {
#   let texture_handle = {
#       let loader = world.read_resource::<Loader>();
#       let texture_storage = world.read_resource::<AssetStorage<Texture>>();
#       loader.load(
#           "texture/pong_spritesheet.png",
#           PngFormat,
#           Default::default(),
#           (),
#           &texture_storage,
#       )
#   };
# let texture_id = 0;
# let mut material_texture_set = world.write_resource::<MaterialTextureSet>();
# material_texture_set.insert(texture_id, texture_handle);  
// Create the sprite for the paddles.
//
// Texture coordinates are expressed as a proportion of the sprite sheet's
// dimensions between 0.0 and 1.0, so they must be divided by the width or
// height.
//
// In addition, on the Y axis, texture coordinates are 0.0 at the bottom of
// the sprite sheet and 1.0 at the top, which is the opposite direction of
// pixel coordinates, so we have to invert the value by subtracting the
// pixel proportion from 1.0.
let tex_coords = TextureCoordinates {
    left: 0.0,
    right: PADDLE_WIDTH / SPRITESHEET_SIZE.0,
    bottom: 1.0 - PADDLE_HEIGHT / SPRITESHEET_SIZE.1,
    top: 1.0,
};
let paddle_sprite = Sprite {
    width: PADDLE_WIDTH,
    height: PADDLE_HEIGHT,
    offsets: [PADDLE_WIDTH / 2.0, PADDLE_HEIGHT / 2.0],
    tex_coords,
};

// Collate the sprite layout information into a sprite sheet
// `sprite_sheet` is the layout of the sprites on the image
let sprite_sheet = SpriteSheet {
    texture_id,
    sprites: vec![paddle_sprite],
};

let sprite_sheet_handle = {
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load_from_data(sprite_sheet, (), &sprite_sheet_store)
};

sprite_sheet_handle
# }
```

That's quite a bit of code, and well deserving of the comments. In the
`TextureCoordinates` set up, the `SPRITESHEET_SIZE.0` and `SPRITESHEET_SIZE.1`
correspond to the pixel width and height of the pong_spritesheet.png file. If
you take a look at the sprite sheet, you will find the paddle sprite on the
left, and the ball on the right. The texture coordinates may be confusing at
first, but the flipped Y axis is how textures have been referenced since the
beginning of time<sup>TM</sup>. From another perspective, it's consistent with
the mathematical graph Y axis, and it is pixel coordinates that are actually
inverted.

Anyway, in the `Sprite` declaration, the width and height indicate how many
pixels the paddle sprite normally takes up &mdash; since the information is not
retained in the `TextureCoordinates`. The `offsets` indicate how many pixels the
sprite should be shifted left and down, relative to the entity that it is
attached to. By using half the width and height of the paddle, the sprite will
be placed such that its center aligns with the middle of the paddle.

Lastly, we construct the `SpriteSheet`, passing it the ID of the texture that
holds its pixel data, and the paddle sprite. The order of sprites declared on
the sprite sheet is also significant, as sprites are referenced by the index in
the vector. If you're wondering about the ball sprite, it does exist on the
image, but we will get to it in a later part of the tutorial.

Phew! That's a lot to take in; but it's not over yet! We have to link the sprite
to the paddle. We update the `initialise_paddle` function by changing its
signature to:

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::World;
# use amethyst::renderer::SpriteSheetHandle;
fn initialise_paddles(world: &mut World, sprite_sheet: SpriteSheetHandle)
# { }
```

Inside `initialise_paddles`, we construct a `SpriteRender` for each paddle.

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::World;
# use amethyst::renderer::{SpriteSheetHandle, SpriteRender};
# fn initialise_paddles(world: &mut World, sprite_sheet: SpriteSheetHandle) {
// Assign the sprites for the paddles
let sprite_render_left = SpriteRender {
    sprite_sheet: sprite_sheet.clone(),
    sprite_number: 0, // paddle is the first sprite in the sprite_sheet
    flip_horizontal: false,
    flip_vertical: false,
};

let sprite_render_right = SpriteRender {
    sprite_sheet: sprite_sheet,
    sprite_number: 0,
    flip_horizontal: true,
    flip_vertical: false,
};
# }
```

`SpriteRender` is the `Component` that indicates which sprite of which sprite
sheet should be drawn for a particular entity. Since the paddle is the first
sprite in the sprite sheet, we use `0` for the `sprite_number`.

Next we simply add the components to the paddle entities:

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::World;
# use amethyst::renderer::{SpriteSheetHandle, SpriteRender};
# use amethyst::prelude::*;
# fn initialise_paddles(world: &mut World, sprite_sheet: SpriteSheetHandle) {
# let sprite_render_left = SpriteRender {
#   sprite_sheet: sprite_sheet.clone(),
#   sprite_number: 0, // paddle is the first sprite in the sprite_sheet
#   flip_horizontal: false,
#   flip_vertical: false,
# };
# let sprite_render_right = SpriteRender {
#   sprite_sheet: sprite_sheet,
#   sprite_number: 0,
#   flip_horizontal: true,
#   flip_vertical: false,
# }; 
// Create a left plank entity.
world
    .create_entity()
    .with(sprite_render_left)
    // ... other components
    .build();

// Create right plank entity.
world
    .create_entity()
    .with(sprite_render_right)
    // ... other components
    .build();
# }
```

We're nearly there, we just have to wire up the sprite to the paddles. We put it
all together in the `on_start()` method:

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::*;
# use amethyst::renderer::{TextureHandle, SpriteSheetHandle};
# use amethyst::ecs::World;
# struct Paddle;
# impl amethyst::ecs::Component for Paddle {
#   type Storage = amethyst::ecs::VecStorage<Paddle>;    
# }
# fn initialise_paddles(world: &mut World, spritesheet: SpriteSheetHandle) { }
# fn initialise_camera(world: &mut World) { }
# fn load_sprite_sheet(world: &mut World) -> SpriteSheetHandle { unimplemented!() }
# struct MyState;
# impl<'a, 'b> SimpleState<'a,'b> for MyState {
fn on_start(&mut self, data: StateData<GameData>) {
    let world = data.world;

    // Load the spritesheet necessary to render the graphics.
    let sprite_sheet_handle = load_sprite_sheet(world);

    world.register::<Paddle>();

    initialise_paddles(world, sprite_sheet_handle);
    initialise_camera(world);
}
# }
```

And we're done. Let's run our game and have fun!

If all is well, we should get something that looks like this:

![Step two](../images/pong_tutorial/pong_02.png)

In the next chapter we'll explore the "S" in ECS and actually get these paddles
moving!

[sb]: https://slide-rs.github.io/specs/
[sb-storage]: https://slide-rs.github.io/specs/05_storages.html#densevecstorage
[cg]: https://docs.rs/cgmath/0.15.0/cgmath/
[2d]: https://www.amethyst.rs/doc/develop/doc/amethyst_renderer/struct.Camera.html#method.standard_2d
