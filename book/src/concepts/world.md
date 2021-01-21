# World

## What is a `World`?

A `World` is a container for entities, with some helper functions that make your life easier.
This chapter will showcase those functions and their usage.  Along with the `World` there is a concept of `Resources` which
are singletons available across all systems.  See [Resources][res] for more about them.

## Creating entities

```rust
# use amethyst::ecs::{World};

#[derive(Default)]
struct MyComponent {
    value: u8,
};

# fn main() {
   let mut world = World::default();
    world.push((MyComponent::default(),));
# }
```

## Accessing a `Component`

```rust
# use amethyst::ecs::{World};
# #[derive(Default)]
# struct MyComponent {
#     value: u8,
# };
# fn main() {
   let mut world = World::default();
    // Create an `Entity` with `MyComponent`.
    // `World` will implicitly write to the component's storage in `Resources`.
    let my_entity =     world.push((MyComponent::default(),));

    if let Some(entry) = world.entry(my_entity) {
        let my_component = entry.get_component::<MyComponent>().unwrap();
    }
# }
```

## Modifying a `Component`

This is almost the same as accessing a component:

```rust
# use amethyst::ecs::{ World};
# #[derive(Default)]
# struct MyComponent {
#     value: u8,
# };
# fn main() {
  let mut world = World::default();
    let my_entity =     world.push((MyComponent::default(),));
    if let Some(entry) = world.entry(my_entity) {
        let mut my_component = entry.get_component_mut::<MyComponent>().unwrap();
        my_component.value = 5;
    }
# }
```

## Delete an entity

Single:

```rust
# use amethyst::ecs::World;
# fn main() {
    let mut world = World::default();
    let my_entity = world.push((MyComponent,));
    assert!(world.remove(my_entity));
# }
```

All:

```rust
# use amethyst::ecs::World;
# fn main() {
#   let mut world = World::default();
    world.clear();
# }
```

__Note: Entities are lazily deleted, which means that deletion only happens at the end of the frame and not immediately when calling the `delete` method.__

## Check if the entity was deleted

```rust
# use amethyst::ecs::World;
# fn main() {
#   let mut world = World::default();
    let my_entity = world.push((0usize,));
    assert!(world.contains(my_entity));
    assert!(!world.contains(Entity(100)))
# }
```

[res]: ./resource.html
