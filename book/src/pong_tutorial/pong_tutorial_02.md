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
use amethyst::prelude::*;
use amethyst::ecs::{Component, DenseVecStorage};
use amethyst::assets::Loader;
use amethyst::core::cgmath::Vector3;
use amethyst::core::transform::{LocalTransform, Transform};
use amethyst::renderer::{Camera, Material, MaterialDefaults, PosTex, MeshHandle, 
                         Event,KeyboardInput, VirtualKeyCode, WindowEvent};
```

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

## Where is the world?

Now it's time to add a new method to our State implementation: `on_start`. 
Inside this function, we'll `register` our `Paddle` component on the mutable 
`World` object we're passed by Amethyst's state machine when the game starts up.

```rust,ignore
fn on_start(&mut self, world: &mut World) {
    world.register::<Paddle>();
}
```

This `World` gets passed around everywhere. It carries with it all the 
components in our game. Not only the components we create, but the ones the 
Amethyst engine itself relies on. For instance, in our `main.rs` we added a 
`RenderBundle::new()` to our game before calling `run()`. That added default 
rendering components like `Camera`, `Material`, and `Mesh` to the `World`, some 
of which we'll be using soon.

## Initialise some entities

Now that we have a Paddle component, let's define some paddle entities that 
include that component and add them to our `World`.

First let's look at our math imports:

```rust,ignore
use amethyst::core::cgmath::Vector3;
use amethyst::core::transform::{LocalTransform, Transform};
```

Amethyst uses the [cgmath crate][cg] under the hood and exposes it for our use. 
Today we just grabbed the `Vector3` type, which is a very good math thing to have.

`LocalTransform` and `Transform` are Amethyst ECS components which carry 
position and orientation information. `LocalTransform` is relative 
to a parent if one exists, while `Transform` is global.

Let's also define some constants for convenience:

```rust,ignore
const PADDLE_HEIGHT: f32 = 0.30;
const PADDLE_WIDTH: f32 = 0.05;
const PADDLE_COLOUR: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
```

Okay, let's make some entities! We'll define an `initialise_paddles` function 
which will create left and right paddle entities and attach `LocalTransform` 
components to them to position them in our world. Our canvas goes from 
`-1.0,-1.0` on the bottom left to `1.0,1.0` on the top right, which will make 
more sense when we define the camera.

```rust,ignore
/// Initialises one paddle on the left, and one paddle on the right.
fn initialise_paddles(world: &mut World) {

    let mut left_transform = LocalTransform::default();
    let mut right_transform = LocalTransform::default();

    // Correctly position the paddles.
    let y = -PADDLE_HEIGHT / 2.0;
    left_transform.translation = Vector3::new(-1.0, y, 0.0);
    right_transform.translation = Vector3::new(1.0 - PADDLE_WIDTH, y, 0.0);

    // Create a left plank entity.
    world
        .create_entity()
        .with(Paddle::new(Side::Left))
        .with(Transform::default())
        .with(left_transform)
        .build();

    // Create right plank entity.
    world
        .create_entity()
        .with(Paddle::new(Side::Right))
        .with(Transform::default())
        .with(right_transform)
        .build();
}
```

This is all the information Amethyst needs to track and move the paddles in our 
virtual world, but we'll need to do some more work to actually *draw* them.

## Drawing

Here's a utility function to generate the six vertices of a rectangle (a 
rectangle in computer graphics is typically drawn with two triangles):

```rust,ignore
fn generate_rectangle_vertices(left: f32,
                               bottom: f32,
                               right: f32,
                               top: f32) -> Vec<PosTex> {
    vec![
        PosTex {
            position: [left, bottom, 0.],
            tex_coord: [0.0, 0.0],
        },
        PosTex {
            position: [right, bottom, 0.0],
            tex_coord: [1.0, 0.0],
        },
        PosTex {
            position: [left, top, 0.0],
            tex_coord: [1.0, 1.0],
        },
        PosTex {
            position: [right, top, 0.],
            tex_coord: [1.0, 1.0],
        },
        PosTex {
            position: [left, top, 0.],
            tex_coord: [0.0, 1.0],
        },
        PosTex {
            position: [right, bottom, 0.0],
            tex_coord: [0.0, 0.0],
        },
    ]
}
```

`PosTex` is a type defined by `amethyst_renderer`. It's a vertex format with 
position and UV texture coordinate attributes. In our rendering pipeline, if 
you'll recall, we created a `DrawFlat::<PosTex>` pass, which draws a `PosTex` 
mesh. Right now our vertices are simply in a standard Rust `Vector`. To create a 
mesh from them we'll write another utility function, which generates a 
`MeshHandle`:

```rust,ignore
fn create_mesh(world: &World, vertices: Vec<PosTex>) -> MeshHandle {
    let loader = world.read_resource::<Loader>();
    loader.load_from_data(vertices.into(), (), &world.read_resource())
}
```

The `Loader` is an asset loader which is defined as a `resource` (not a `Entity`
, `Component`, or `System`, but still a part of our ECS `world`). It was created 
when we built our Application in `main.rs`, and it can read assets like .obj 
files, but also it can `load_from_data` as in our use case.

> Resources in Specs are a type of data which can be shared between systems, 
> while being independent from entities, in contrast to components, which are 
> attached to specific entities. We'll explore this more later on.

The `load_from_data` function returns a `Handle<Mesh>`, also known as a 
`MeshHandle`. Since `Handle` implements component, we can attach it to our 
entity. Once the mesh is fully loaded, a system which asks the handle for the 
mesh will receive it. If the mesh isn't loaded yet, the handle will return 
`None`. In this minimal scenario, the mesh will be available on the next 
frame.

In addition to mesh data, we also need a material to draw our mesh with. 
We'll use the Amethyst renderer's `MaterialDefaults` (another resource) and only 
change the albedo color:

```rust,ignore
/// Creates a solid material of the specified colour.
fn create_colour_material(world: &World, colour: [f32; 4]) -> Material {
    let mat_defaults = world.read_resource::<MaterialDefaults>();
    let loader = world.read_resource::<Loader>();

    let albedo = loader.load_from_data(colour.into(),
                                       (),
                                       &world.read_resource());

    Material {
        albedo,
        ..mat_defaults.0.clone()
    }
}
```

Now let's return to inside our `initialise_paddles` function and actually create 
this mesh and material.

```rust,ignore
let mesh = create_mesh(
    world,
    generate_rectangle_vertices(0.0, 0.0, PADDLE_WIDTH, PADDLE_HEIGHT),
);

let material = create_colour_material(world, PADDLE_COLOUR);
```

Now we just add these components to our paddle entities:

```rust,ignore
// Create a left plank entity.
world
    .create_entity()
    .with(mesh.clone())
    .with(material.clone())
    .with(Paddle::new(Side::Left))
    .with(Transform::default())
    .with(left_transform)
    .build();

// Create right plank entity.
world
    .create_entity()
    .with(mesh)
    .with(material)
    .with(Paddle::new(Side::Right))
    .with(Transform::default())
    .with(right_transform)
    .build();
```

We're almost done! We just need to create a camera to view all our beautiful 
graphics (two blue pong paddles) with.

```rust,ignore
fn initialise_camera(world: &mut World) {
    world
        .create_entity()
        .with(Camera::standard_2d())
        .build();
}
```

The camera is another entity, with a `Camera` component. Amethyst's 
[`standard_2d` camera][2d] uses an orthographic projection, and defines a 
screenspace coordinate system of `-1.0,-1.0` in the bottom left and `1.0,1.0` in 
the top right.

If you want to to define your own coordinate system, you can use something like 
this:

```rust,ignore
fn initialise_camera(world: &mut World) {
    world.create_entity()
        .with(Camera::from(Projection::orthographic(0.0, WIDTH, HEIGHT, 0.0)))
        .with(Transform(Matrix4::from_translation
                (Vector3::new(0.0, 0.0, 1.0)).into())
             )
        .build();
}
```

To use that custom camera you'll need to define WIDTH and HEIGHT constants, and 
redo the position math in the `initialise_paddles` function.

Now let's add our initialise functions to the `on_start` function in `impl State 
for Pong`.

```rust,ignore
fn on_start(&mut self, world: &mut World) {
    world.register::<Paddle>();
    initialise_paddles(world);
    initialise_camera(world);
}
```

Okay! We've defined our `Paddle` component, and created two entities which have 
`Paddle`, `Transform`, `MeshHandle`, and `Material` components. When our game 
starts, we'll register the `Paddle` component and then add the left and right 
paddles to the world, along with a camera.

Let's run this and see what happens. On my machine I get a panic that reads:

```
No component with the given id. Did you forget to register the component with 
`World::register::<ComponentName>()`?
```

It looks like we're missing at least one component registration. In addition to 
components we define ourselves, Amethyst has a lot of internal systems and 
components it uses to keep things running. For simplicity, these have been 
wrapped up into "Bundles" which include related components, systems, and 
resources. We can add these to our Application using the `with_bundle` method, 
and in fact we already have one of these in `main.rs`: the `RenderBundle`.

As it turns out, the components we're missing are `Transform` and 
`LocalTransform`, and we can add those with the `TransformBundle`, which will 
also add the `TransformSystem` for working with those components:

```rust,ignore
let mut game = Application::build("./", Pong)?
    .with_bundle(TransformBundle::new())? //Add this bundle
    .with_bundle(RenderBundle::new())?
    .with_local(RenderSystem::build(pipe, Some(config))?)
    .build()?;
```

Also we'll need to import that structure:

```rust,ignore
use amethyst::core::transform::TransformBundle;
```

Now when we run the game we should get something that looks like this:

![Step two](./images/pong_tutorial/pong_02.png)

In the next chapter we'll explore the "S" in ECS and actually get these paddles 
moving!

[sb]: https://slide-rs.github.io/specs/
[sb-storage]: https://slide-rs.github.io/specs/05_storages.html#densevecstorage
[cg]: https://docs.rs/cgmath/0.15.0/cgmath/
[2d]: https://www.amethyst.rs/doc/develop/doc/amethyst_renderer/struct.Camera.html#method.standard_2d
