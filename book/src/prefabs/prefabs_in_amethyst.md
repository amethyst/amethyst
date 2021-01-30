# Prefabs in Amethyst

> **Note:** This page assumes you have read and understood [assets].

Many game engines – including Amethyst – treat prefabs as [assets], and so they are usually stored as files and loaded at runtime. After loading the asset(s), prefabs have additional processing to turn them into [`Component`]s and attach them to entities.

## Representation

There are two representations of a prefab:

- Stored representation, distributed alongside the application.
- Loaded representation, used at runtime to instantiate entities with components.

The remainder of this page explains these at a conceptual level; subsequent pages contain guides on how Amethyst applies this at a code level.

### The Basics

> **Note:** The prefab examples on this page include the [`PrefabData`] type names. These are written out for clarity. However, as per the RON specification, these are not strictly required.

In its stored form, a prefab is a serialized list of entities and their components that should be instantiated together. To begin, we will look at a simple prefab that attaches a simple component to a single entity. We will use the following `Position` component:

```rust
# extern crate derivative;
# extern crate serde;
# 
# use amethyst::{
#   assets::{Prefab, PrefabData},
#   derive::PrefabData,
#   ecs::Entity,
#   prelude::*,
#   Error,
# };
# use derivative::Derivative;
# use serde::{Deserialize, Serialize};
# 
#[derive(Clone, Copy, Component, Debug, Default, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
# #[serde(deny_unknown_fields)]
pub struct Position(pub f32, pub f32, pub f32);
```

The important derives are the [`Component`] and [`PrefabData`] – [`Component`] means it can be attached to an entity; [`PrefabData`] means it can be loaded as part of a prefab. The `#[prefab(Component)]` attribute informs the [`PrefabData`] derive that this type is a [`Component`], as opposed to being composed of fields which implement [`PrefabData`]. This will only be important when implementing a custom prefab.

Here is an example `.ron` file of a prefab with an entity with a `Position`:

```rust
#![enable(implicit_some)]
Prefab(
    entities: [
        PrefabEntity(
            // parent: None // Optional
            data: Position(1.0, 2.0, 3.0),
        ),
    ],
)
```

The top level type is a [`Prefab`], and holds a list of `entities`. These are not the [`Entity`] type used at runtime, but the [`PrefabEntity`] type – a template for what [`Component`]s to attach to entities at runtime. Each of these holds two pieces of information:

- `data`: Specifies the [`Component`]s to attach to the entity.

  This must be a type that implements [`PrefabData`]. When this prefab is instantiated, it will attach a `Position` component to the entity.

- `parent`: (Optional) index of this entity's [`Parent`] entity. The value is the index of the parent entity which resides within this prefab file.

When we load this prefab, the prefab entity is read as:

```rust
PrefabEntity { parent: None, data: Some(Position(1.0, 2.0, 3.0)) }
```

Next, we create an entity with the prefab handle, `Handle<Prefab<Position>>`:

| Entity                   | Handle\<Prefab\<Position>> |
| ------------------------ | -------------------------- |
| Entity(0, Generation(1)) | Handle { id: 0 }           |

In the background, the [`PrefabLoaderSystem`] will run, and attach the `Position` component:

| Entity                   | Handle\<Prefab\<Position>> | Position                |
| ------------------------ | -------------------------- | ----------------------- |
| Entity(0, Generation(1)) | Handle { id: 0 }           | Position(1.0, 2.0, 3.0) |

This can be seen by running the `prefab` example from the Amethyst repository:

```bash
cargo run -p prefab
```

### Multiple Components

If there are multiple components to be attached to the entity, then we need a type that aggregates the [`Component`]s:

```rust
# extern crate derivative;
# extern crate serde;
# 
# use amethyst::{
#   assets::{Prefab, PrefabData, ProgressCounter},
#   core::Named,
#   derive::PrefabData,
#   ecs::Entity,
#   prelude::*,
#   Error,
# };
# use derivative::Derivative;
# use serde::{Deserialize, Serialize};
# 
# #[derive(Clone, Copy, Component, Debug, Default, Deserialize, Serialize, PrefabData)]
# #[prefab(Component)]
# #[serde(deny_unknown_fields)]
# pub struct Position(pub f32, pub f32, pub f32);
# 
#[derive(Debug, Deserialize, Serialize, PrefabData)]
# #[serde(deny_unknown_fields)]
pub struct Player {
    player: Named,
    position: Position,
}
```

Here, the `Player` type is **not** a [`Component`], but it does implement [`PrefabData`]. Each of its fields is a [`PrefabData`] as well as a [`Component`].

The corresponding prefab file is written as follows:

```rust
#![enable(implicit_some)]
Prefab(
    entities: [
        PrefabEntity(
            data: Player(
                player: Named(name: "Zero"),
                position: Position(1.0, 2.0, 3.0),
            ),
        ),
    ],
)
```

When an entity is created with this prefab, Amethyst will recurse into each of the prefab data fields – [`Named`] and `Position` – to attach their respective components to the entity.

Now, when we create an entity with the prefab handle, both components will be attached:

| Handle\<Prefab\<Player>> | Position                | Player                 |
| ------------------------ | ----------------------- | ---------------------- |
| Handle { id: 0 }         | Position(1.0, 2.0, 3.0) | Named { name: "Zero" } |

This can be seen by running the `prefab_multi` example from the Amethyst repository:

```bash
cargo run -p prefab_multi
```

### Multiple Entities, Different Components

The next level is to instantiate multiple entities, each with their own set of [`Component`]s. The current implementation of [`Prefab`] requires the `data` field to be the same type for *every* [`PrefabEntity`] in the list. This means that to have different types of entity in the same prefab they must be variants of an enum. For instance, a prefab like this:

```rust
#![enable(implicit_some)]
Prefab(
    entities: [
        // Player
        PrefabEntity(
            data: Player(
                player: Named(name: "Zero"),
                position: Position(1.0, 2.0, 3.0),
            ),
        ),
        // Weapon
        PrefabEntity(
            parent: 0,
            data: Weapon(
                weapon_type: Sword,
                position: Position(4.0, 5.0, 6.0),
            ),
        ),
    ],
)
```

Could be implemented using an enum like this:

```rust
# extern crate derivative;
# extern crate serde;
# 
# use amethyst::{
#   assets::{Prefab, PrefabData, ProgressCounter},
#   core::Named,
#   derive::PrefabData,
#   ecs::Entity,
#   prelude::*,
#   utils::application_root_dir,
#   Error,
# };
# use derivative::Derivative;
# use serde::{Deserialize, Serialize};
# 
# #[derive(Clone, Copy, Component, Debug, Default, Deserialize, Serialize, PrefabData)]
# #[prefab(Component)]
# #[serde(deny_unknown_fields)]
# pub struct Position(pub f32, pub f32, pub f32);
# 
#[derive(Clone, Copy, Component, Debug, Derivative, Deserialize, Serialize, PrefabData)]
#[derivative(Default)]
#[prefab(Component)]
pub enum Weapon {
    #[derivative(Default)]
    Axe,
    Sword,
}

#[derive(Debug, Deserialize, Serialize, PrefabData)]
#[serde(deny_unknown_fields)]
pub enum CustomPrefabData {
    Player {
        name: Named,
        position: Option<Position>,
    },
    Weapon {
        weapon_type: Weapon,
        position: Option<Position>,
    },
}
```

When we run this, we start off by creating one entity:

| Entity                   | Handle\<Prefab\<CustomPrefabData>>> |
| ------------------------ | ----------------------------------- |
| Entity(0, Generation(1)) | Handle { id: 0 }                    |

When the [`PrefabLoaderSystem`] runs, this becomes the following:

| Entity                   | Handle\<Prefab\<CustomPrefabData>>> | Parent                   | Position                | Player                 | Weapon |
| ------------------------ | ----------------------------------- | ------------------------ | ----------------------- | ---------------------- | ------ |
| Entity(0, Generation(1)) | Handle { id: 0 }                    | None                     | Position(1.0, 2.0, 3.0) | Named { name: "Zero" } | None   |
| Entity(1, Generation(1)) | None                                | Entity(0, Generation(1)) | Position(4.0, 5.0, 6.0) | None                   | Sword  |

- The entity that the `Handle<Prefab<T>>` is attached will be augmented with [`Component`]s from the first [`PrefabEntity`].
- A new entity is created for subsequent [`PrefabEntity`] entries in the `entities` list.

Note that the `Weapon` has a parent with index `0`. Let's see what happens when multiple entities are created with this prefab. First, two entities are created with the prefab handle:

| Entity                   | Handle\<Prefab\<CustomPrefabData>>> |
| ------------------------ | ----------------------------------- |
| Entity(0, Generation(1)) | Handle { id: 0 }                    |
| Entity(1, Generation(1)) | Handle { id: 0 }                    |

Next, the [`PrefabLoaderSystem`] runs and creates and augments the entities:

| Entity                   | Handle\<Prefab\<CustomPrefabData>>> | Parent                   | Position                | Player                 | Weapon |
| ------------------------ | ----------------------------------- | ------------------------ | ----------------------- | ---------------------- | ------ |
| Entity(0, Generation(1)) | Handle { id: 0 }                    | None                     | Position(1.0, 2.0, 3.0) | Named { name: "Zero" } | None   |
| Entity(1, Generation(1)) | Handle { id: 0 }                    | None                     | Position(1.0, 2.0, 3.0) | Named { name: "Zero" } | None   |
| Entity(2, Generation(1)) | None                                | Entity(0, Generation(1)) | Position(4.0, 5.0, 6.0) | None                   | Sword  |
| Entity(3, Generation(1)) | None                                | Entity(1, Generation(1)) | Position(4.0, 5.0, 6.0) | None                   | Sword  |

The sword entity `2` has player entity `0` as its parent, and sword entity `3` has player entity `1` as its parent.

This can be seen by running the `prefab_custom` example from the Amethyst repository:

```bash
cargo run -p prefab_custom
```

______________________________________________________________________

Phew, that was long! Now that you have an understanding of how prefabs work in Amethyst, the next page covers the technical aspects in more detail.

[assets]: ../assets.html
[`component`]: https://docs.rs/specs/~0.16/specs/trait.Component.html
[`entity`]: https://docs.rs/specs/~0.16/specs/struct.Entity.html
[`named`]: https://docs.amethyst.rs/master/amethyst_core/struct.Named.html
[`parent`]: https://docs.amethyst.rs/master/amethyst_core/transform/components/struct.Parent.html
[`prefabdata`]: https://docs.amethyst.rs/master/amethyst_assets/trait.PrefabData.html#impl-PrefabData
[`prefabentity`]: https://github.com/amethyst/amethyst/blob/v0.15.3/amethyst_assets/src/prefab/mod.rs#L121-L126
[`prefabloadersystem`]: https://docs.amethyst.rs/master/amethyst_assets/struct.PrefabLoaderSystem.html
[`prefab`]: https://docs.amethyst.rs/master/amethyst_assets/struct.Prefab.html
