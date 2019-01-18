# Prefabs in Amethyst

> **Note:** This page assumes you have read and understood [assets].

Many game engines &ndash; including Amethyst &ndash; treat prefabs as [assets], and so they are usually stored as files and loaded at runtime. After loading the asset(s), prefabs have additional processing to turn them into [`Component`]s and attach them to entities.

## Representation

There are two representations of a prefab:

* Stored representation, distributed alongside the application.
* Loaded representation, used at runtime to instantiate entities with components.

The remainder of this page explains these at a conceptual level; subsequent pages contain guides on how Amethyst applies this at a code level.

### The Basics

> **Note:** The prefab examples on this page include the [`PrefabData`] type names. These are written out for clarity. However, as per the RON specification, these are not strictly required.

In its stored form, a prefab is a serialized list of entities and their components that should be instantiated together. To begin, we will look at a simple prefab that attaches a simple component to a single entity. We will use the following `Position` component:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# extern crate derivative;
# extern crate serde;
# extern crate specs_derive;
#
# use amethyst::{
#     assets::{Prefab, PrefabData, PrefabError},
#     derive::PrefabData,
#     ecs::{
#         storage::DenseVecStorage,
#         Component, Entity, WriteStorage,
#     },
#     prelude::*,
# };
# use derivative::Derivative;
# use serde::{Deserialize, Serialize};
# use specs_derive::Component;
#
#[derive(Clone, Copy, Component, Debug, Default, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
# #[serde(deny_unknown_fields)]
pub struct Position(pub f32, pub f32, pub f32);
```

The important derives are the [`Component`] and [`PrefabData`] &ndash; [`Component`] means it can be attached to an entity; [`PrefabData`] means it can be loaded as part of a prefab. The `#[prefab(Component)]` attribute informs the [`PrefabData`] derive that this type is a [`Component`], as opposed to being composed of fields which implement [`PrefabData`]. This will only be important when implementing a custom prefab.

Here is an example `.ron` file of a prefab with an entity with a `Position`:

```rust,ignore
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

The top level type is a [`Prefab`], and holds a list of `entities`. These are not the [`Entity`] type used at runtime, but the [`PrefabEntity`] type &ndash; a template for what [`Component`]s to attach to entities at runtime. Each of these holds two pieces of information:

* `data`: Specifies the [`Component`]s to attach to the entity.

    This must be a type that implements [`PrefabData`]. When this prefab is instantiated, it will attach a `Position` component to the entity.

* `parent`: (Optional) index of this entity's [`Parent`] entity. The value is the index of the parent entity which resides within this prefab file.

When we load this prefab, the prefab entity is read as:

```rust,ignore
PrefabEntity { parent: None, data: Some(Position(1.0, 2.0, 3.0)) }
```

Next, we create an entity with the prefab handle, `Handle<Prefab<Position>>`:

| Entity                   | Handle<Prefab&lt;Position>> |
| ------------------------ | ------------------------ |
| Entity(0, Generation(1)) | Handle { id: 0 }         |

In the background, the [`PrefabLoaderSystem`] will run, and attach the `Position` component:

| Entity                   | Handle<Prefab&lt;Position>> | Position                |
| ------------------------ | ------------------------ | ----------------------- |
| Entity(0, Generation(1)) | Handle { id: 0 }         | Position(1.0, 2.0, 3.0) |

This can be seen by running the `prefab_basic` example from the Amethyst repository:

```bash
cargo run --example prefab_basic
```

### Multiple Components

If there are attach multiple components to be attached to the entity, then we need a type that aggregates the [`Component`]s:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# extern crate derivative;
# extern crate serde;
# extern crate specs_derive;
#
# use amethyst::{
#     assets::{Prefab, PrefabData, PrefabError, ProgressCounter},
#     core::Named,
#     derive::PrefabData,
#     ecs::{
#         storage::{DenseVecStorage, VecStorage},
#         Component, Entity, WriteStorage,
#     },
#     prelude::*,
# };
# use derivative::Derivative;
# use serde::{Deserialize, Serialize};
# use specs_derive::Component;
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

```rust,ignore
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

When an entity is created with this prefab, Amethyst will recurse into each of the prefab data fields &ndash; [`Named`] and `Position` &ndash; to attach their respective components to the entity.

Now, when we create an entity with the prefab handle, both components will be attached:

| Handle<Prefab&lt;Player>> | Position                | Player                 |
| ---------------------- | ----------------------- | ---------------------- |
| Handle { id: 0 }       | Position(1.0, 2.0, 3.0) | Named { name: "Zero" } |

This can be seen by running the `prefab_multi` example from the Amethyst repository:

```bash
cargo run --example prefab_multi
```

### Multiple Entities, Different Components

The next level is to instantiate multiple entities, each with their own set of [`Component`]s. The current implementation of [`Prefab`] requires the `data` field to be the same type for *every* [`PrefabEntity`] in the list. This means we would be unable to declare something like the following snippet, because it uses a `Player` prefab data in one entity, and `Weapon` in another:

```rust,ignore
// Note: Invalid / erroneous example
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
                type: Sword,
                position: Position(4.0, 5.0, 6.0),
            ),
        ),
    ],
)
```

Instead, the components have to be moved up to a top level [`PrefabData`] type, with components wrapped in an [`Option`]. In the following snippet, the top level [`PrefabData`] is `CustomPrefabData`:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# extern crate derivative;
# extern crate serde;
# extern crate specs_derive;
#
# use amethyst::{
#     assets::{Prefab, PrefabData, PrefabError, ProgressCounter},
#     core::Named,
#     derive::PrefabData,
#     ecs::{
#         storage::{DenseVecStorage, VecStorage},
#         Component, Entity, WriteStorage,
#     },
#     prelude::*,
#     utils::application_root_dir,
#     Error,
# };
# use derivative::Derivative;
# use serde::{Deserialize, Serialize};
# use specs_derive::Component;
#
# #[derive(Clone, Copy, Component, Debug, Default, Deserialize, Serialize, PrefabData)]
# #[prefab(Component)]
# #[serde(deny_unknown_fields)]
# pub struct Position(pub f32, pub f32, pub f32);
#
#[derive(Clone, Copy, Component, Debug, Derivative, Deserialize, Serialize, PrefabData)]
#[derivative(Default)]
#[prefab(Component)]
#[storage(VecStorage)]
pub enum Weapon {
    #[derivative(Default)]
    Axe,
    Sword,
}

#[derive(Debug, Default, Deserialize, Serialize, PrefabData)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct CustomPrefabData {
    player: Option<Named>,
    weapon: Option<Weapon>,
    position: Option<Position>,
}
```

The prefab is then declared like so:

```rust,ignore
#![enable(implicit_some)]
Prefab(
    entities: [
        // Player
        PrefabEntity(
            data: CustomPrefabData(
                player: Player(name: "Zero"),
                position: Position(1.0, 2.0, 3.0),
            ),
        ),
        // Weapon
        PrefabEntity(
            parent: 0,
            data: CustomPrefabData(
                weapon: Sword,
                position: Position(4.0, 5.0, 6.0),
            ),
        ),
    ],
)
```

When we run this, we start off by creating one entity:

| Entity                   | Handle<Prefab&lt;CustomPrefabData>>> |
| ------------------------ | --------------------------------- |
| Entity(0, Generation(1)) | Handle { id: 0 }                  |

When the [`PrefabLoaderSystem`] runs, this becomes the following:

| Entity                   | Handle<Prefab&lt;CustomPrefabData>>> | Parent                   | Position                | Player                 | Weapon |
| ------------------------ | --------------------------------- | ------------------------ | ----------------------- | ---------------------- | ------ |
| Entity(0, Generation(1)) | Handle { id: 0 }                  | None                     | Position(1.0, 2.0, 3.0) | Named { name: "Zero" } | None   |
| Entity(1, Generation(1)) | None                              | Entity(0, Generation(1)) | Position(4.0, 5.0, 6.0) | None                   | Sword  |

* The entity that the `Handle<Prefab<T>>` is attached will be augmented with [`Component`]s from the first [`PrefabEntity`].
* A new entity is created for subsequent [`PrefabEntity`] entries in the `entities` list.

Note that the `Weapon` has a parent with index `0`. Let's see what happens when multiple entities are created with this prefab. First, two entities are created with the prefab handle:

| Entity                   | Handle<Prefab&lt;CustomPrefabData>>> |
| ------------------------ | --------------------------------- |
| Entity(0, Generation(1)) | Handle { id: 0 }                  |
| Entity(1, Generation(1)) | Handle { id: 0 }                  |

Next, the [`PrefabLoaderSystem`] runs and creates and augments the entities:

| Entity                   | Handle<Prefab&lt;CustomPrefabData>>> | Parent                   | Position                | Player                 | Weapon |
| ------------------------ | --------------------------------- | ------------------------ | ----------------------- | ---------------------- | ------ |
| Entity(0, Generation(1)) | Handle { id: 0 }                  | None                     | Position(1.0, 2.0, 3.0) | Named { name: "Zero" } | None   |
| Entity(1, Generation(1)) | Handle { id: 0 }                  | None                     | Position(1.0, 2.0, 3.0) | Named { name: "Zero" } | None   |
| Entity(2, Generation(1)) | None                              | Entity(0, Generation(1)) | Position(4.0, 5.0, 6.0) | None                   | Sword  |
| Entity(3, Generation(1)) | None                              | Entity(1, Generation(1)) | Position(4.0, 5.0, 6.0) | None                   | Sword  |

The sword entity `2` has player entity `0` as its parent, and sword entity `3` has player entity `1` as its parent.

This can be seen by running the `prefab_custom` example from the Amethyst repository:

```bash
cargo run --example prefab_custom
```

---

Phew, that was long! Now that you have an understanding of how prefabs work in Amethyst, the next page covers the technical aspects in more detail.

[assets]: ../assets.html
[`Component`]: https://www.amethyst.rs/doc/latest/doc/specs/trait.Component.html
[`Entity`]: https://www.amethyst.rs/doc/latest/doc/specs/struct.Entity.html
[`Named`]: https://www.amethyst.rs/doc/latest/doc/amethyst_core/struct.Named.html
[`Option`]: https://doc.rust-lang.org/std/option/enum.Option.html
[`Parent`]: https://www.amethyst.rs/doc/latest/doc/amethyst_core/transform/components/struct.Parent.html
[`Prefab`]: https://www.amethyst.rs/doc/latest/doc/amethyst_assets/struct.Prefab.html
[`PrefabData`]: https://www.amethyst.rs/doc/latest/doc/amethyst_assets/trait.PrefabData.html#impl-PrefabData%3C%27a%3E
[`PrefabEntity`]: https://github.com/amethyst/amethyst/blob/v0.10.0/amethyst_assets/src/prefab/mod.rs#L110-L115
[`PrefabLoaderSystem`]: https://www.amethyst.rs/doc/latest/doc/amethyst_assets/struct.PrefabLoaderSystem.html
