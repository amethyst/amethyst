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

You first start by creating the entity builder.
Then, you can add components to your entity.
Finally, you call the build() method on the entity builder to get the actual entity.

```rust,ignore
    let mut entity_builder = world.create_entity();
    entity_builder = entity_builder.with(MyComponent);
    let my_entity = entity_builder.build();
```

Shorter version:
```rust,ignore
    let my_entity = world
       .create_entity()
       .with(MyComponent)
       .build();
```

Internally, the `World` interacts with `EntitiesRes`, which is a resource holding the entities inside of `Resources`.

## Accessing a `Component`

```rust,ignore
    // Create an `Entity` with `MyComponent`.
    // `World` will implicitly write to the component's storage in `Resources`.
    let my_entity = world.create_entity().with(MyComponent).build();
    
    // Get a ReadStorage<MyComponent>
    let storage = world.read_storage::<MyComponent>();
    
    // Get the actual component from the storage.
    let my = storage.get(&my_entity).expect("Failed to get component for entity: {:?}", my_entity);
```

## Modifying a `Component`

This is almost the same as accessing a component:

```rust,ignore
    let my_entity = world.create_entity().with(MyComponent).build();
    let mut storage = world.write_storage::<MyComponent>();
    let mut my = storage.get_mut(&my_entity).expect("Failed to get component for entity: {:?}", my_entity);
```

## Getting all entities

It is pretty rare to use this, but can be useful in some occasions.

```rust,ignore
    // Returns `EntitiesRes`
    let entities = world.entities();
```

## Delete an entity

Single:
```rust,ignore
    world.delete_entity(my_entity).expect("Failed to delete entity. Was it already removed?");
```

Multiple:
```rust,ignore
    world.delete_entities(entity_vec.as_slice()).expect("Failed to delete entities from specified list.");
```

All:
```rust,ignore
    world.delete_all().expect("Failed to delete all entities.");
```

## Check if the entity was deleted

```rust,ignore
    // Returns true if the entity was **not** deleted.
    let is_alive = world.is_alive(&my_entity);
```

## Exec

**This is just to show that this feature exists. It is normal to not understand what it does until you read the system chapter**

Sometimes, you will want to create an entity where you need to fetch resources to create the correct components for it.
There is a function that acts as a shorthand for this:

```rust,ignore
    world
        .exec(|mut data: SomeSystemData| {
            data.do_something();
        })
    .expect("Error creating SpriteRender for paddles");
```

We will talk about what `SystemData` is in the system chapter.

## More

Because of an implementation detail, for the world to be updated you need to call `state_data.data.update(&state_data.world)` inside of the state update method.
If you do not do this, the world will not update and the game will look like it froze.
If you have a bug where nothing happens at all, this is the first thing you should check!

*NOTE: This will no longer be necessary in future release 0.9 of Amethyst.*