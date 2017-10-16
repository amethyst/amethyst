# The Game System

In this stage we are going to add a system to our world that will run the main
game mechanics. We won't implement everything, but we will see how the system
reads the state and updates it.

First thing we want to do is create some components that contain our game data,
rather than render/input. So far we've use built-in components, this is the
first time we write them ourselves.

```rust
struct Ball {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub size: f32,
}

impl Ball {
    pub fn new() -> Ball {
        Ball {
            position: [0.0, 0.0],
            velocity: [-1.0, -1.0],
            size: 1.0,
        }
    }
}

impl Component for Ball {
    type Storage = VecStorage<Ball>;
}
```

Let's go through what we've done here. Firstly, we create a datatype for our
ball. It contains 2D position and velocity information, and a size, and has a
simple constructor.

We then implement `Component` for our data structure, allowing it to be used as
a component in our *ecs* world. To do this, we need to say how we store the
data. `VecStorage` is probably the simplest way, and according to the
documentation optimal for dense data (components present on most entities).
Since we're only going to have 3 entities (2 paddles and a ball) it probably
doesn't matter about performance.

We now want to attach this component to our existing ball entity.

```rust
// Create a ball entity
let mut ball = Ball::new();
ball.size = 0.02;
ball.velocity = [0.5, 0.5];
world.create_now()
    .with(square.clone())
    .with(ball)
    .with(LocalTransform::default())
    .with(Transform::default())
    .build();
```
You can see we simply add an extra component.

Finally, we need to register this component when we create our application.

```rust
let mut game = Application::build(Pong, cfg)
    .register::<Ball>()
    .done();
```

Now we can run the example again. There won't be any difference since the
previous stage because although we have a new component, there is no system to
link it to the rendering component. Let's change that!

```rust
struct PongSystem;

// Pong game system
impl System<()> for PongSystem {
    fn run(&mut self, arg: RunArg, _: ()) {
        use amethyst::ecs::resources::{Camera, InputHandler, Time};
        // Get all needed component storages and resources
        let (mut balls, locals, camera, time, input) = arg.fetch(|w| {
                (w.write::<Ball>(),
                 w.write::<LocalTransform>(),
                 w.read_resource::<Camera>(),
                 w.read_resource::<Time>(),
                 w.read_resource::<InputHandler>())
            });
    }
}
```

Here we're adding our custom `System`. Eventually, it will do everything from
moving the ball to keeping the score, but for now we'll just make it draw the
ball in the correct screen position given its world position and size.

Above you can see how a system is created. We define a marker type (a type with
no data, like `Pong` earlier), and then implement the `System` trait. This
trait has a single method, `run`, that represents the system.

The method contains an argument called `RunArg`. This object provides methods
to access components and resources from the ecs, and locking them. *TODO is this
true?* It's
important to lock components for as short a time as possible, so that more work
can go on in parallel.

When we read/write a resource, we get a read/write lock immediately, whereas we
are required to lock components manually using the `pass` method.

We can also see `Time` here, a resource we haven't seen before. We'll be using
it for `delta_time`, which is updated by amethyst with the duration since `run`
was last called. This is used to maintain consistent updates given variation in
the time between frames. see `std::time::Duration` from the standard library
for more info.
