# How to Define Prefabs: Aggregate

This guide explains how to define a [`PrefabData`] that encapsulates other [`PrefabData`].

If you intend to include a [`Component`] that has not yet got a corresponding [`PrefabData`], please use an appropriate guide from the [available guides][bk_prefab_prelude] to create its [`PrefabData`] first.

## Steps

1. Ensure your crate has the following dependencies:

   ```toml
   [dependencies]
   amethyst = ".." # Minimum version 0.10
   serde = { version = "1", features = ["derive"] }
   ```

1. Import the following items:

   ```rust
   use amethyst::{
       assets::{PrefabData, ProgressCounter},
       derive::PrefabData,
       ecs::Entity,
       Error,
   };
   use serde::{Deserialize, Serialize};
   ```

1. Define the aggregate prefab data type.

   In these examples, `Named`, `Position`, and `Weapon` all derive [`PrefabData`].

   ```rust
   # extern crate serde;
   # use amethyst::{
   #   assets::{PrefabData, ProgressCounter},
   #   core::Named,
   #   derive::PrefabData,
   #   ecs::Entity,
   #   prelude::*,
   #   Error,
   # };
   # use serde::{Deserialize, Serialize};
   # 
   #[derive(Clone, Copy, Component, Debug, Default, Deserialize, Serialize, PrefabData)]
   #[prefab(Component)]
   #[serde(deny_unknown_fields)]
   pub struct Position(pub f32, pub f32, pub f32);

   /// **Note:** All fields must be specified in the prefab. If a field is
   /// not specified, then the prefab will fail to load.
   #[derive(Deserialize, Serialize, PrefabData)]
   #[serde(deny_unknown_fields)]
   pub struct Player {
       name: Named,
       position: Position,
   }
   ```

   If you want to mix different types of entities within a single prefab then you must define an enum that implements `PrefabData`. Each variant is treated in the same way as `PrefabData` structs.

   ```rust
   # extern crate serde;
   # use amethyst::{
   #   assets::{PrefabData, ProgressCounter},
   #   core::Named,
   #   derive::PrefabData,
   #   ecs::Entity,
   #   prelude::*,
   #   Error,
   # };
   # use serde::{Deserialize, Serialize};
   # 
   #[derive(Clone, Copy, Component, Debug, Default, Deserialize, Serialize, PrefabData)]
   #[prefab(Component)]
   #[serde(deny_unknown_fields)]
   pub struct Position(pub f32, pub f32, pub f32);

   #[derive(Clone, Copy, Component, Debug, Deserialize, Serialize, PrefabData)]
   #[prefab(Component)]
   pub enum Weapon {
       Axe,
       Sword,
   }

   /// All fields implement `PrefabData`.
   ///
   /// **Note:** If a field is of type `Option<_>` and not specified in the prefab, it will default
   /// to `None`.
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

   **Note:** There is an important limitation when building `PrefabData`s, particularly enum `PrefabData`s. No two fields in the `PrefabData` or in any nested `PrefabData`s under it can access the same `Component` unless all accesses are reads. This is still true even if the fields appear in different variants of an enum. This means that the following `PrefabData` will fail at runtime when loaded:

   ```rust
   # extern crate serde;
   # use amethyst::{
   #   assets::{PrefabData, ProgressCounter},
   #   core::Named,
   #   derive::PrefabData,
   #   ecs::Entity,
   #   prelude::*,
   #   renderer::sprite::prefab::SpriteScenePrefab,
   #   Error,
   # };
   # use serde::{Deserialize, Serialize};

   #[derive(Clone, Copy, Component, Debug, Default, Deserialize, Serialize, PrefabData)]
   #[prefab(Component)]
   #[serde(deny_unknown_fields)]
   pub struct SpecialPower;

   #[derive(Debug, Deserialize, Serialize, PrefabData)]
   #[serde(deny_unknown_fields)]
   pub enum CustomPrefabData {
       MundaneCreature {
           sprite: SpriteScenePrefab,
       },
       MagicalCreature {
           special_power: SpecialPower,
           sprite: SpriteScenePrefab,
       },
   }
   ```

   The problem is that both the `SpriteScenePrefab`s need to write to `Transform` and several other common `Components`. Because Amythest's underlyng ECS system determines what resources are accessed based on static types it can't determine that only one of the `SpriteScenePrefab`s will be accessed at a time and it attempts a double mutable borrow which fails. The solution is to define the `PrefabData` hierarchically so each component only appears once:

   ```rust
   # extern crate serde;
   # use amethyst::{
   #   assets::{PrefabData, ProgressCounter},
   #   core::Named,
   #   derive::PrefabData,
   #   ecs::Entity,
   #   prelude::*,
   #   renderer::sprite::prefab::SpriteScenePrefab,
   #   Error,
   # };
   # use serde::{Deserialize, Serialize};

   #[derive(Clone, Copy, Component, Debug, Default, Deserialize, Serialize, PrefabData)]
   #[prefab(Component)]
   #[serde(deny_unknown_fields)]
   pub struct SpecialPower;

   #[derive(Debug, Deserialize, Serialize, PrefabData)]
   #[serde(deny_unknown_fields)]
   pub enum CreatureDetailsPrefab {
       MundaneCreature {},
       MagicalCreature { special_power: SpecialPower },
   }
   #[derive(Debug, Deserialize, Serialize, PrefabData)]
   #[serde(deny_unknown_fields)]
   pub struct CustomPrefabData {
       sprite: SpriteScenePrefab,
       creature_details: CreatureDetailsPrefab,
   }
   ```

   The [`PrefabData`][api_pf_derive] derive implements the [`PrefabData`] trait for the type. The generated code will handle invoking the appropriate [`PrefabData`] methods when loading and attaching components to an entity. **Note:** This differs from the simple component [`PrefabData`] derive implementation – there is no `#[prefab(Component)]` attribute.

   The [`#[serde(default)]`][ser_def] attribute allows fields to not be specified in the prefab, and the fields' default value will be used. If this attribute is not present, then all fields must be specified in the prefab.

   Finally, the [`#[serde(deny_unknown_fields)]`][ser_unk] ensures that deserialization produces an error if it encounters an unknown field. This will help expose mistakes in the prefab file, such as when there is a typo.

1. Now the type can be used in a prefab.

   - `struct` prefab data:

     ```rust
     #![enable(implicit_some)]
     Prefab(
         entities: [
             PrefabEntity(
                 data: Player(
                     name: Named(name: "Zero"),
                     position: Position(1.0, 2.0, 3.0),
                 ),
             ),
         ],
     )
     ```

   - `enum` prefab data:

     ```rust
     #![enable(implicit_some)]
     Prefab(
         entities: [
             // Player
             PrefabEntity(
                 data: Player(
                     name: Named(name: "Zero"),
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

To see this in a complete example, run the [`prefab_custom` example][repo_prefab_custom] or the [`prefab_multi` example][repo_prefab_multi] from the Amethyst repository:

```bash
cargo run -p prefab_custom # superset prefab
cargo run -p prefab_multi # object prefab
```

[api_pf_derive]: https://docs.amethyst.rs/master/amethyst_derive/derive.PrefabData.html
[bk_prefab_prelude]: how_to_define_prefabs_prelude.html
[repo_prefab_custom]: https://github.com/amethyst/amethyst/tree/master/examples/prefab_custom
[repo_prefab_multi]: https://github.com/amethyst/amethyst/tree/master/examples/prefab_multi
[ser_def]: https://serde.rs/container-attrs.html#default
[ser_unk]: https://serde.rs/container-attrs.html#deny_unknown_fields
[`component`]: https://docs.rs/specs/~0.16/specs/trait.Component.html
[`prefabdata`]: https://docs.amethyst.rs/master/amethyst_assets/trait.PrefabData.html
