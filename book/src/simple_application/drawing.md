
# Setting Up An ECS System

In this stage, we are going to draw something!

> You will be able to run your game during this stage, but you will need to
> `Ctrl+c` the terminal window to stop it, as we don't process keyboard events
> yet.

Firstly, we need to revisit the `State.onStart` trait method.

```rust
fn on_start(&mut self, 
            world: &mut World, 
            assets: &mut AssetManager, 
            pipe: &mut Pipeline)
```
Let's talk about the parameters here. The first one - `&mut World` is the
global object containing all entities, components, and systems. We can use it to
add *entities* to the world. These entities can have an arbitrary set of
*component* information associated with them, The entity is the thing, and the
components are the bits of data attached to it (or more accurately, the types
of information). *Systems* then drive the data in the entities for the
components they are interested in/responsible for. I find it a bit confusing,
so let's jump into pong to see if a real world example helps.

When we want to add an entity, we call `world.create_now()` which returns a
[builder pattern][builder_pattern] object. We can then call `with` on this
object to add component information, and then when we have added everything we
want to, we call `build` to finalise it into an entity in our world.
Here is a full example for our pong ball.

```rust
world.create_now()
    .with(square.clone())
    .with(ball)
    .with(LocalTransform::default())
    .with(Transform::default())
    .build();
```
Some of the components we can see here are built into amethyst to control how
things are drawn, and we can add an unlimited number of our own components.
We'll see how some of the built-in components work later.

# Creating a ball entity

We're going to spend the rest of this stage learning how to draw to the screen.
We want to create an entity, and attach the required component information to
it so amethyst can draw it.

Let's look at the second parameter to `on_start`, a `&mut AssetManager`. The
asset manager is responsible for holding rendering information, including
meshes and textures. We're going to tell it to make space for our meshes and
textures, then to store some very simple ones that we need to draw our ball. We
will use a flat color for our texture, so there will be no need to load any
image files, and we will use a simple square for our ball, containing only 2
triangles.

## Adding a renderable

```rust

// Generate a square mesh
assets.register_asset::<Mesh>();
assets.register_asset::<Texture>();
assets.load_asset_from_data::<Texture, [f32; 4]>("white", [1.0, 1.0, 1.0, 1.0]);
let square_verts = gen_rectangle(1.0, 1.0);
assets.load_asset_from_data::<Mesh, Vec<VertexPosNormal>>("square", square_verts);
let square = assets.create_renderable("square", "white", "white", "white", 1.0).unwrap();
```

The stages are
 1. Register asset types, so we can store assets of that type
 2. Load a single pixel texture into our asset manager. The texture is 4 `f32`s
    wide, one for each of rgba. It is labelled with the string "white".
 3. Generate vertices, normals and texture coordinates for a square. This is
    not partiularly specific to amethyst, so I'm not going to explain it all in
    detail, but very briefly there are 2 triangles of 3 vertices each. Normals
    all point up, and texture triangles are over our single pixel. The asset is
    labelled "square".
 4. Finally, we create a *renderable* out of the mesh and texture. This is
    something we can draw, essentially just a block of data. We will use this 
    to draw all the objects in our scene (they will all be white rectangles. 
    We refer to assets by their string label. See the method signature in the 
    docs for what the parameters are.

## Creating the pipeline

In amethyst (and probably elsewhere, I'm not an expert), the pipeline detials
the number and order of the rendering passes. We will have a single pass,
consisting of a layer to clear the previous frame, and a layer to draw our ball 
(and paddles, added later).

```rust
let layer = Layer::new("main",
                       vec![Clear::new([0.0, 0.0, 0.0, 1.0]),
                            DrawFlat::new("main", "main")]);
pipe.layers.push(layer);
```

We use the `DrawFlat` program because we don't need to apply shading (we're
basically in 2D). The first "main" in `DrawFlat` is the name of the camera we
are using, and the second is the name of the scene we want to draw. The camera
and scene are added for us by amethyst, but we will want to set the camera's
direction.

## View and projection matrices (a.k.a. the Camera)

```rust
// I start a code block so we release the resources after we've finished with
// them (camera).
{
    // As per, we use aspect ratio to generate our projection so our view
    // doesn't get squashed.
    let dim = world.read_resource::<ScreenDimensions>().pass();
    let mut camera = world.write_resource::<Camera>().pass();
    let aspect_ratio = dim.aspect_ratio;
    let eye = [0., 0., 0.1];
    let target = [0., 0., 0.];
    let up = [0., 1., 0.];

    // Get an Orthographic projection
    let proj = Projection::Orthographic {
        left: -1.0 * aspect_ratio,
        right: 1.0 * aspect_ratio,
        bottom: -1.0,
        top: 1.0,
        near: 0.0,
        far: 1.0,
    };

    // The projection matrix (made for us by `Projection::Orthographic` :) )
    camera.proj = proj;
    // Where the camera is
    camera.eye = eye;
    // What it's looking at
    camera.target = target;
    // Which way is up
    camera.up = up;
}
```

There's nothing particularly unusual here, we just set up our view and
projection matrices. Currently this is done by setting some properties on the
camera.

We can now run our game if we like, but we will still just see a blank screen.
Let's change that!

```rust

// Create a ball entity
world.create_now()
    .with(square.clone())
    .with(LocalTransform::default())
    .with(Transform::default())
    .build();
```
And now, if we do `cargo run`, we get a white square!!

It's worth explaining what's going on here. We create an entity and give it 3
components: the `Renderable` component, a built-in that allows something to be
drawn (we created this near the beginning), the `Transform` component,
that we use to [transform][quaternion] elements relative to the global origin,
and `InnerTransform` that we use to transform our entity relative to its
parent. Later, we will give it more components that we write to manage our game
world, and add a system for updating these, and linking them to the rendering
components above.

 [ecs]: ../glossary.md#entity-component-system-ecs-model
 [builder_pattern]: https://en.wikipedia.org/wiki/Builder_pattern
 [quaternion]: https://en.wikipedia.org/wiki/Quaternions_and_spatial_rotation
