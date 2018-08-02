# Component

## What is a `Component`?

A component is any struct that can be "attached" to an `Entity` (we will cover entity in the next chapter).

## Usage

The relationship between an entity and a component closely ressembles the relation between a real-life object and its properties.

For example, a bottle of water has a shape, a volume, a color and is made of a material(usually plastic).

In this example, the bottle is the entity, and the properties are components.

## Creating a component

Creating a component is easy.

You declare a struct:

```rust
pub struct MyComponent{
    some_property: String,
}
```

and then you implement the `Component` trait for it.

```rust,ignore
use amethyst::ecs::{Component, VecStorage};
impl Component for MyComponent {
    type Storage = VecStorage<Self>;
}
```

## Storages?

`Component`s, in contrast with popular belief, should not be stored directly inside of a `Entity`.

They are instead stored in different types of `Storage`, which all have different performance strategies.

When implementing `Component` for a type, you have to specify which storage strategy it should use.

Here's a comparison of the most used ones:
* `DenseVecStorage`: Elements are stored in a contiguous array. No empty space is left between `Component`s, allowing a lowered memory usage.
  This is less performant than `VecStorage`.
* `VecStorage`: Elements are stored into a contiguous array. Uses more memory than `DenseVecStorage` but is more performant when you use a `Component` type a lot.
  Only use this when you know that a LOT of your entities are going to have that `Component` type. (More than 90%, for example).
* `FlaggedStorage`: Used to keep track of changes of a component. Useful for caching purposes.

For more information, see the [specs storage reference](https://docs.rs/specs/latest/specs/storage/index.html).

## Tags

Components can also be used to "tag" entities.
The usual way to do it is to create an empty struct, and implement `Component` using `NullStorage` as the `Storage` type for it.
Null storage means that it is not going to take memory space to store those components.

You will learn how to use those tag components in the system chapter.
