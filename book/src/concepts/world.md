# World

## What is a world?

A world is just a holder for `Resources`, with some helper functions that make your life easier.
This chapter will showcase those functions and their usage.

## Adding a resource

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
use amethyst::ecs::World;

// A simple struct with no data.
struct MyResource;

fn main() {
    // We create a new `World` instance.
    let mut world = World::new();
    
    // We create our resource.
    let my = MyResource;
    
    // We add the resource to the world.
    world.add_resource(my);
}
```

## Fetching a resource

Here's how to fetch a read-only resource. Be aware that this method panics if the resource isn't inserted into `Resources`.
```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# struct MyResource;
# fn main() {
#   let mut world = amethyst::ecs::World::new();
    let my = world.read_resource::<MyResource>();
# }
```

If you are not sure that the resource will be present, use the methods available on `Resources`, as shown in the resource chapter.
```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# struct MyResource;
# fn main() {
#   let mut world = amethyst::ecs::World::new();
    let my = world.res.entry::<MyResource>().or_insert_with(|| MyResource);
# }
```

## Modifying a resource

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# struct MyResource;
# fn main() {
#   let mut world = amethyst::ecs::World::new();
    let mut my = world.write_resource::<MyResource>();
# }
```

## Creating entities

You first start by creating the entity builder.
Then, you can add components to your entity.
Finally, you call the build() method on the entity builder to get the actual entity.
Please note that **in order to use this syntax, you need to import the ``amethyst::prelude::Builder`` trait.**

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# struct MyComponent;
# impl amethyst::ecs::Component for MyComponent {
#   type Storage = amethyst::ecs::VecStorage<MyComponent>;
# }
# fn main() {
#   let mut world = amethyst::ecs::World::new();
    use amethyst::prelude::Builder;

    let mut entity_builder = world.create_entity();
    entity_builder = entity_builder.with(MyComponent);
    let my_entity = entity_builder.build();
# }
```

Shorter version:
```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# struct MyComponent;
# impl amethyst::ecs::Component for MyComponent {
#   type Storage = amethyst::ecs::VecStorage<MyComponent>;
# }
# fn main() {
#   let mut world = amethyst::ecs::World::new();
    use amethyst::prelude::Builder;

    let my_entity = world
       .create_entity()
       .with(MyComponent)
       .build();
# }
```

Internally, the `World` interacts with `EntitiesRes`, which is a resource holding the entities inside of `Resources`.

## Accessing a `Component`

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::Builder;
# struct MyComponent;
# impl amethyst::ecs::Component for MyComponent {
#   type Storage = amethyst::ecs::VecStorage<MyComponent>;
# }
# fn main() {
#   let mut world = amethyst::ecs::World::new();
    // Create an `Entity` with `MyComponent`.
    // `World` will implicitly write to the component's storage in `Resources`.
    let my_entity = world.create_entity().with(MyComponent).build();
    
    // Get a ReadStorage<MyComponent>
    let storage = world.read_storage::<MyComponent>();
    
    // Get the actual component from the storage.
    let my = storage.get(my_entity).expect("Failed to get component for entity");
# }
```

## Modifying a `Component`

This is almost the same as accessing a component:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::Builder;
# struct MyComponent;
# impl amethyst::ecs::Component for MyComponent {
#   type Storage = amethyst::ecs::VecStorage<MyComponent>;
# }
# fn main() {
#   let mut world = amethyst::ecs::World::new();
    let my_entity = world.create_entity().with(MyComponent).build();
    let mut storage = world.write_storage::<MyComponent>();
    let mut my = storage.get_mut(my_entity).expect("Failed to get component for entity");
# }
```

## Getting all entities

It is pretty rare to use this, but can be useful in some occasions.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# fn main() {
#   let mut world = amethyst::ecs::World::new();
    // Returns `EntitiesRes`
    let entities = world.entities();
# }
```

## Delete an entity

Single:
```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::Builder;
# fn main() {
#   let mut world = amethyst::ecs::World::new();
#   let my_entity = world.create_entity().build();
    world.delete_entity(my_entity).expect("Failed to delete entity. Was it already removed?");
# }
```

Multiple:
```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::Builder;
# fn main() {
#   let mut world = amethyst::ecs::World::new();
#   let entity_vec: Vec<amethyst::ecs::Entity> = vec![world.create_entity().build()];
    world.delete_entities(entity_vec.as_slice()).expect("Failed to delete entities from specified list.");
# }
```

All:
```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# fn main() {
#   let mut world = amethyst::ecs::World::new();
    world.delete_all();
# }
```

__Note: Entities are lazily deleted, which means that deletion only happens at the end of the frame and not immediately when calling the `delete` method.__

## Check if the entity was deleted

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::prelude::Builder;
# fn main() {
#   let mut world = amethyst::ecs::World::new();
#   let my_entity = world.create_entity().build();
    // Returns true if the entity was **not** deleted.
    let is_alive = world.is_alive(my_entity);
# }
```

## Exec

**This is just to show that this feature exists. It is normal to not understand what it does until you read the system chapter**

Sometimes, you will want to create an entity where you need to fetch resources to create the correct components for it.
There is a function that acts as a shorthand for this:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::ecs::ReadExpect;
# struct Dummy;
# type SomeSystemData<'a> = ReadExpect<'a, Dummy>;
# trait DoSomething {
#   fn do_something(&mut self);
# }
# impl<'a> DoSomething for SomeSystemData<'a> {
#   fn do_something(&mut self) { }
# }
# fn main() {
#   let mut world = amethyst::ecs::World::new();
    world.exec(|mut data: SomeSystemData| {
        data.do_something();
    });
# }
```

We will talk about what `SystemData` is in the system chapter.
