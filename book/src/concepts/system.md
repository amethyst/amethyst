# System

## What is a `System`?

A system is where the logic of the game is executed. In practice, it consists of a struct implementing a function executed on every iteration of the game loop, and taking as an argument data about the game.

Systems can be seen as a small unit of logic. All systems are run by the engine together (even in parallel when possible), and do a specialized operation on one or a group of entities.

## Structure

A system struct is a structure implementing the trait `amethyst::ecs::System`.

Here is a very simple example implementation:

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::System;
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

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::{System, Read};
# use amethyst::core::timing::Time;
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

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::{Entity, System, WriteStorage};
# use amethyst::core::Transform;
struct WalkPlayerUp {
    player: Entity,
}

impl<'a> System<'a> for WalkPlayerUp {
    type SystemData = WriteStorage<'a, Transform>;

    fn run(&mut self, mut transforms: Self::SystemData) {
        transforms.get_mut(self.player).unwrap().translate_y(0.1);
    }
}
```

This system makes the player go up by 0.1 unit every iteration of the game loop! To identify what entity the player is, we stored it beforehand in the system's struct. Then, we get its `Transform` from the transform storage, and move it along the Y axis by 0.1.

> A transform is a very common structure in game development. It represents the position, rotation and scale of an object in the game world. You will use them a lot, as they are what you need to change when you want to move something around in your game.

However, this approach is pretty rare because most of the time you don't know what entity you want to manipulate, and in fact you may want to apply your changes to multiple entities.

### Getting all entities with specific components

Most of the time, you will want to perform logic on all entities with a specific components, or even all entities with a selection of components.

This is possible using the `join` method. You may be familiar with joining operations if you have ever worked with databases. The `join` method takes multiple storages, and iterates over all entities that have a component in each of those storages.
It works like an "AND" gate. It will return an iterator containing a tuple of all the requested components if they are **ALL** on the same entity.

If you join with components A, B and C, only the entities that have **ALL** those components will be considered.

Needless to say that you can use it with only one storage to iterate over all entities with a specific component.

Keep in mind that **the `join` method is only available by importing `amethyst::ecs::Join`**.

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::{System, ReadStorage, WriteStorage};
# use amethyst::core::Transform;
# struct FallingObject;
# impl amethyst::ecs::Component for FallingObject {
#   type Storage = amethyst::ecs::DenseVecStorage<FallingObject>;
# }
use amethyst::ecs::Join;

struct MakeObjectsFall;

impl<'a> System<'a> for MakeObjectsFall {
    type SystemData = (
        WriteStorage<'a, Transform>,
        ReadStorage<'a, FallingObject>,
    );

    fn run(&mut self, (mut transforms, falling): Self::SystemData) {
        for (mut transform, _) in (&mut transforms, &falling).join() {
            if transform.translation().y > 0.0 {
                transform.translate_y(-0.1);
            }
        }
    }
}
```

This system will make all entities with both a `Transform` with a positive y coordinate and a `FallingObject` tag component fall by 0.1 unit per game loop iteration. Note that as the `FallingObject` is only here as a tag to restrict the joining operation, we immediately discard it using the `_` syntax.

Cool! Now that looks like something we'll actually do in our games!

### Getting entities that have some components, but not others

There is a special type of `Storage` in specs called `AntiStorage`.
The not operator (!) turns a Storage into its AntiStorage counterpart, allowing you to iterate over entities that do NOT have this `Component`.
It is used like this:

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::{System, ReadStorage, WriteStorage};
# use amethyst::core::Transform;
# struct FallingObject;
# impl amethyst::ecs::Component for FallingObject {
#   type Storage = amethyst::ecs::DenseVecStorage<FallingObject>;
# }
use amethyst::ecs::Join;

struct NotFallingObjects;

impl<'a> System<'a> for NotFallingObjects {
    type SystemData = (
        WriteStorage<'a, Transform>,
        ReadStorage<'a, FallingObject>,
    );

    fn run(&mut self, (mut transforms, falling): Self::SystemData) {
        for (mut transform, _) in (&mut transforms, !&falling).join() {
            // If they don't fall, why not make them go up!
            transform.translate_y(0.1);
        }
    }
}
```

## Manipulating the structure of entities

It may sometimes be interesting to manipulate the structure of entities in a system, such as creating new ones or modifying the component layout of existing ones. This kind of process is done using the `Entities<'a>` system data.

> Requesting `Entities<'a>` does not impact performance, as it contains only immutable resources and therefore [does not block the dispatching](./dispatcher.html).

### Creating new entities in a system

Creating an entity while in the context of a system is very similar to the way one would create an entity using the `World` struct. The only difference is that one needs to provide mutable storages of all the components they plan to add to the entity.

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::{System, WriteStorage, Entities};
# use amethyst::core::Transform;
# struct Enemy;
# impl amethyst::ecs::Component for Enemy {
#   type Storage = amethyst::ecs::VecStorage<Enemy>;
# }
struct SpawnEnemies {
    counter: u32,
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
```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::{System, Entities, Entity};
# struct MySystem { entity: Entity }
# impl<'a> System<'a> for MySystem {
#   type SystemData = Entities<'a>;
#   fn run(&mut self, entities: Self::SystemData) {
#       let entity = self.entity;
entities.delete(entity);
#   }
# }
```

### Iterating over components with associated entity

Sometimes, when you iterate over components, you may want to also know what entity you are working with. To do that, you can use the joining operation with `Entities<'a>`.

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::{Join, System, Entities, WriteStorage, ReadStorage};
# use amethyst::core::Transform;
# struct FallingObject;
# impl amethyst::ecs::Component for FallingObject {
#   type Storage = amethyst::ecs::VecStorage<FallingObject>;
# }
struct MakeObjectsFall;

impl<'a> System<'a> for MakeObjectsFall {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, FallingObject>,
    );

    fn run(&mut self, (entities, mut transforms, falling): Self::SystemData) {
        for (e, mut transform, _) in (&*entities, &mut transforms, &falling).join() {
            if transform.translation().y > 0.0 {
                transform.translate_y(-0.1);
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

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::{System, Entities, Entity, WriteStorage};
# struct MyComponent;
# impl amethyst::ecs::Component for MyComponent {
#   type Storage = amethyst::ecs::VecStorage<MyComponent>;
# }
# struct MySystem { entity: Entity }
# impl<'a> System<'a> for MySystem {
#   type SystemData = WriteStorage<'a, MyComponent>;
#   fn run(&mut self, mut write_storage: Self::SystemData) {
#       let entity = self.entity;
// Add the component
write_storage.insert(entity, MyComponent);

// Remove the component
write_storage.remove(entity);
#   }
# }
```

Keep in mind that inserting a component on an entity that already has a component of the same type **will overwrite the previous one**.

## The SystemData trait

While this is rarely useful, it is possible to create custom `SystemData` types.

The `Dispatcher` populates the `SystemData` on every call of the `run` method. To do that, your `SystemData` type must implement the trait `amethyst::ecs::SystemData` in order to have it be valid.

This is rather complicated trait to implement, fortunately Amethyst provides a derive macro for it, that can implement the trait to any struct as long as all its fields are `SystemData`. Most of the time however, you will not even need to implement it at all as you will be using `SystemData` structs provided by the engine.

Please note that tuples of structs implementing `SystemData` are themselves `SystemData`. This is very useful when you need to request multiple `SystemData` at once quickly.

```rust,no_run,noplaypen
# extern crate amethyst;
# extern crate shred;
# #[macro_use] extern crate shred_derive;
# 
# use amethyst::ecs::{ReadStorage, WriteStorage, SystemData, Component, VecStorage, System, Join};
#
# struct FooComponent {
#   stuff: f32,
# }
# impl Component for FooComponent {
#   type Storage = VecStorage<FooComponent>;
# }
# 
# struct BarComponent {
#   stuff: f32,
# }
# impl Component for BarComponent {
#   type Storage = VecStorage<BarComponent>;
# }
# 
# #[derive(SystemData)]
# struct BazSystemData<'a> {
#  field: ReadStorage<'a, FooComponent>,
# }
# 
# impl<'a> BazSystemData<'a> {
#   fn should_process(&self) -> bool {
#       true
#   }
# }
#
#[derive(SystemData)]
struct MySystemData<'a> {
    foo: ReadStorage<'a, FooComponent>,
    bar: WriteStorage<'a, BarComponent>,
    baz: BazSystemData<'a>,
}

struct MyFirstSystem;

impl<'a> System<'a> for MyFirstSystem {
    type SystemData = MySystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        if data.baz.should_process() {
            for (foo, mut bar) in (&data.foo, &mut data.bar).join() {
                bar.stuff += foo.stuff;
            } 
        }
    }
}
```

## The setup method

Systems have a method called setup which is called a single time, before any of the system runs.
Here is how to use it:

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::{System, Resources, SystemData, Entity};
# struct MySystem { entity: Entity }
# impl<'a> System<'a> for MySystem {
#   type SystemData = ();
#   fn run(&mut self, _: Self::SystemData) { }
    fn setup(&mut self, res: &mut Resources) {
        // Ensures that resources that implement `Default` and are present in your `SystemData` are added to `Resources`.
        Self::SystemData::setup(res);
        // Do what you want with `Resources` past this point.
    }
# }
```
