# World

## What is a world?

A world is just a holder for `Resources`, with some helper functions that make your life easier.
This chapter will showcase those functions and their usage.

## Adding a resource

```rust,norun
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
```rust,ignore
    let my = world.read_resource::<MyResource>();
```

If you are not sure that the resource will be present, use the methods available on `Resources`, as shown in the resource chapter.
```rust,ignore
    let my = world.res.entry::<MyResource>().or_insert_with(|| MyResource);
```

## Modifying a resource

```rust,ignore
    let mut my = world.write_resource::<MyResource>();
```

## Creating entities

```rust,ignore
   let mut entity_builder = world.create_entity();
   entity_builder.with()
```
