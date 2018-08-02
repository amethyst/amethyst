# System

## What is a `System`?

A system is where the logic of the game is executed. In practice, it consists of a struct implementing a function executed on every iteration of the game loop, and taking as an argument data about the game.

## Structure

A system struct is a structure implementing the trait `amethyst::ecs::System`.

Here is a very simple example implementation:

```rust,ignore
struct MyFirstSystem;

impl<'a> System<'a> for MyFirstSystem {
    type SystemData = ();

    fn run(&mut self, data: Self::SystemData) {
        println!("Hello!");
    }
}
```

This system will, on every iteration of the game loop, print "Hello!" in the console. This is a pretty boring system as it does not interact at all with the game. Let us spice it up a bit.

## Accessing the context of the game

In the definition of a system, the trait requires you to define a type `SystemData`. This type defines what data the system will be provided with on each call of its `run` method. `SystemData` is only meant to carry information accessible to multiple systems. Data local to a system is usually stored in the system's struct itself instead.

The Amethyst engine provides useful system data types to use in order to access the context of a game. Here are some of the most important ones:

* **Read<'a, Resource>** (respectively **Write<'a, Resource>**) allows you to obtain an immutable (respectively mutable) reference to a resource of the type you specify. This is guaranteed to not fail as if the resource is not available, it will give you the ``Default::default()`` of your resource. 
* **ReadExpect<'a, Resource>** (respectively **WriteExpect<'a, Resource>**) is a failable alternative to the previous system data, so that you can use resources that do not implement the `Default` trait.
* **ReadStorage<'a, Component>** (respectively **WriteStorage<'a, Component>**) allows you to obtain an immutable (respectively mutable) reference to the entire storage of a certain `Component` type.
* **Entities<'a>** allows you to create or destroy entities in the context of a system.

You can then use one, or multiple of them via a tuple.

```rust,ignore
struct MyFirstSystem;

impl<'a> System<'a> for MyFirstSystem {
    type SystemData = Read<'a, Time>;

    fn run(&mut self, data: Self::SystemData) {
        println!("{}", data.delta_seconds());
    }
}
```

Here, we get the `amethyst::core::timing::Time` resource to print in the console the time elapsed between two frames. Nice! But that's still a bit boring.

## Manipulating storages

Once you have access to a storage, you can use them in different ways.

### Getting a component of a specific entity

Sometimes, it can be useful to get a component in the storage for a specific entity. This can easily be done using the `get` or, for mutable storages, `get_mut` methods.

```rust,ignore
struct CameraGoesUp;

impl<'a> System<'a> for CameraGoesUp {
    type SystemData = (
        WriteStorage<'a, Transform>,
        Read<'a, ActiveCamera>,
    );

    fn run(&mut self, (mut transforms, camera): Self::SystemData) {
        transforms.get_mut(camera.entity).unwrap().translation.y += 0.1;
    }
}
```

This system makes the current main camera (obtained through the  `amethyst::renderer::ActiveCamera` resource) go up by 0.1 unit every iteration of the game loop!

However, this approach is pretty rare because most of the time you don't know what entity you want to manipulate, and in fact you may want to apply your changes to multiple entities.

### Getting all entities with specific components

Most of the time, you will want to perform logic on all entities with a specific components, or even all entities with a selection of components.

This is possible using the `join` method. You may be familiar with joining operations if you have ever worked with databases. The `join` method takes multiple storages, and iterates over all entities that have a component in each of those storages.
It works like an "AND" gate. It will return an iterator containing a tuple of all the requested components if they are **ALL** on the same entity.

If you join with components A, B and C, only the entities that have **ALL** those components will be considered.

Needless to say that you can use it with only one storage to iterate over all entities with a specific component.

Keep in mind that **the `join` method is only available by importing `amethyst::ecs::Join`**.

```rust,ignore
use amethyst::ecs::Join;

struct MakeObjectsFall;

impl<'a> System<'a> for MakeObjectsFall {
    type SystemData = (
        WriteStorage<'a, Transform>,
        ReadStorage<'a, FallingObject>,
    );

    fn run(&mut self, (mut transforms, falling): Self::SystemData) {
        for (mut transform, _) in (&mut transforms, &falling).join() {
            if transform.translation.y > 0.0 {
                transform.translation.y -= 0.1;
            }
        }
    }
}
```

This system will make all entities with both a `Transform` with a positive y coordinate and a `FallingObject` tag component fall by 0.1 unit per game loop iteration. Note that as the `FallingObject` is only here as a tag to restrict the joining operation, we immediately discard it using the `_` syntax.

Cool! Now that looks like something we'll actually do in our games!

## Manipulating the structure of entities

It may sometimes be interesting to manipulate the structure of entities in a system, such as creating new ones or modifying the component layout of existing ones. This kind of process is done using the `Entities<'a>` system data.

> Requesting `Entities<'a>` does not impact performance, as it contains only immutable resources and therefore [does not block the dispatching](./dispatcher.html).

### Creating new entities in a system

Creating an entity while in the context of a system is very similar to the way one would create an entity using the `World` struct. The only difference is that one needs to provide mutable storages of all the components they plan to add to the entity.

```rust,ignore
struct SpawnEnemies {
    counter: u32
}

impl<'a> System<'a> for SpawnEnemies {
    type SystemData = (
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Enemy>,
        Entities<'a>,
    );

    fn run(&mut self, (mut transforms, mut enemies, entities): Self::SystemData) {
        self.counter += 1;
        if self.counter > 200 {
            entities.build_entity()
                .with(Transform::default(), &mut transforms)
                .with(Enemy, &mut enemies)
                .build();
            self.counter = 0;
        }
    }
}
```

This system will spawn a new enemy every 200 game loop iterations.

### Removing an entity

Deleting an entity is very easy using `Entities<'a>`.
```rust
entities.delete(entity);
```

### Iterating over components with associated entity

Sometimes, when you iterate over components, you may want to also know what entity you are working with. To do that, you can use the joining operation with `Entities<'a>`.

```rust,ignore
struct MakeObjectsFall;

impl<'a> System<'a> for MakeObjectsFall {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, FallingObject>,
    );

    fn run(&mut self, (entities, mut transforms, falling): Self::SystemData) {
        for (e, mut transform, _) in (&*entities, &mut transforms, &falling).join() {
            if transform.translation.y > 0.0 {
                transform.translation.y -= 0.1;
            } else {
                entities.delete(e);
            }
        }
    }
}
```

This system does the same thing as the previous `MakeObjectsFall`, but also cleans up falling objects that reached the ground.

### Adding or removing components

You can also insert or remove components from a specific entity.
To do that, you need to get a mutable storage of the component you want to modify, and simply do:

```rust,ignore
// Add the component
write_storage.insert(entity, MyComponent);

// Remove the component
write_storage.remove(entity);
```

Keep in mind that inserting a component on an entity that already has a component of the same type **will overwrite the previous one**.

## The SystemData trait

While this is rarely useful, it is possible to create custom `SystemData` types.

The `Dispatcher` populates the `SystemData` on every call of the `run` method. To do that, your `SystemData` type must implement the trait `amethyst::ecs::SystemData` in order to have it be valid.

This is rather complicated trait to implement, fortunately Amethyst provides a derive macro for it, that can implement the trait to any struct as long as all its fields are `SystemData`. Most of the time however, you will not even need to implement it at all as you will be using `SystemData` structs provided by the engine.

Please note that tuples of structs implementing `SystemData` are themselves `SystemData`. This is very useful when you need to request multiple `SystemData` at once quickly.

```rust,ignore
#[derive(SystemData)]
struct MySystemData<'a> {
    foo: ReadStorage<'a, FooComponent>,
    bar: WriteStorage<'a, BarComponent>,
    baz: BazSystemData<'a>,
}

struct MyFirstSystem;

impl<'a> System<'a> for MyFirstSystem {
    type SystemData = MySystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        if data.baz.should_process() {
            for (foo, mut bar) in (&data.foo, &mut data.bar) {
                bar.stuff += foo.stuff;
            } 
        }
    }
}
```

## The setup method

Systems have a method called setup which is called a single time, before any of the system runs.
Here is how to use it:
```rust,ignore
    fn setup(&mut self, res: &mut Resources) {
        // Ensures that resources that implement `Default` and are present in your `SystemData` are added to `Resources`.
        Self::SystemData::setup(&mut res);
        // Do what you want with `Resources` here.
    }
```
