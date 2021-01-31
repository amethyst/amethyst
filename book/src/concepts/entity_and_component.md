# Entity and Component

## What are `Entity` and `Component`?

An `Entity` represents a single object in your world. `Component` represents one aspect of an object. For example, a bottle of water has a shape, a volume, a color and is made of a material (usually plastic). In this example, the bottle is the entity, and shape, volume, color and material are components.

## Entity and Component in Amethyst

In an inheritance design, an entity usually contains components. All the data and methods related to an entity are stored within.
However, in ECS design, an entity is a general purpose identifier. In fact, the implementation of `Entity` in Amethyst is simply:

```rust
struct Entity(u64);
```

where the u64 is the id of the entity.

The data associated with the entities are grouped into archetypes.

Consider an example where you have three objects: two bottles and a person.

| archetype |   x   |   y   |  shape   |  color  |  name   |
| :-------: | :---: | :---: | :------: | :-----: | :-----: |
|  Bottle   | 150.0 | 202.1 | "round"  |  "red"  |         |
|  Bottle   | 570.0 | 122.0 | "square" | "white" |         |
|  Person   | 100.5 | 300.8 |          |         | "Peter" |

Entities do not store data, nor do they know any information about their components. They serve the purpose of object identification and tracking object existence.
The archetype storages store all the component data and their connection to entities.

If you are familiar with relational databases, this organization looks quite similar to the tables in a database, where the entity id serves as the key in each table.
In fact, you can even join components and entities like joining tables. For example, to update the position of all the named entities, you will need to query on the `Name` and the `Position` components.

Querying is covered in the systems chapter.

## Declaring a component and creating an Archetype

To declare a component, you declare the relevant underlying data.  Legion ECS will create archetypes that correspond to the different combinations of this data.:

```rust
# use amethyst::{
#   core::{
#       math::{Isometry3, Vector3},
#       Named,
#   },
#   ecs::World,
# };
# 

/// This `Component` describes the shape of an `Entity`
enum Shape {
    Sphere { radius: f32 },
    Cuboid { height: f32, width: f32, depth: f32 },
}

/// This `Component` describes the transform of an `Entity`
#[derive(Default)]
struct Transform {
    /// Translation + rotation value
    iso: Isometry3<f32>,
    /// Scale vector
    scale: Vector3<f32>,
}

fn main() {
    let world = World::default();

    // One archetype of entity
    world.push((Shape::Sphere { radius: 3. }, Transform::default()));

    // Another archetype of entity
    world.push((
        // some components are provided by amethyst
        Named("Cubey".into()),
        Shape::Cuboid {
            height: 4.,
            width: 4.,
            depth: 4.,
        },
        Transform::default(),
    ));
}
```

## Tags

Components can also be used to "tag" entities.
The usual way to do it is to create an empty struct or use the `Tag` component provided by Amethsyt

You will learn how to use those tag components in the System chapter.
