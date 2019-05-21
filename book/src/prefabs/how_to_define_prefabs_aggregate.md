# How to Define Prefabs: Aggregate

This guide explains how to define a [`PrefabData`] that encapsulates other [`PrefabData`]. This pattern primarily caters for the following use cases:

* *Superset* [`PrefabData`] that encapsulates [`PrefabData`] used across all entities in a prefab.
* *Object* [`PrefabData`] that encapsulates multiple [`PrefabData`] that must exist together.

This can be applied when the encapsulating prefab data is independent of the child prefab datas, and is simply serving as a combinational type &ndash; an aggregate.

If you intend to include a [`Component`] that has not yet got a corresponding [`PrefabData`], please use an appropriate guide from the [available guides][bk_prefab_prelude] to create its [`PrefabData`] first.

## Steps

1. Ensure your crate has the following dependencies:

    ```toml
    [dependencies]
    amethyst = ".." # Minimum version 0.10
    serde = { version = "1.0", features = ["derive"] }
    ```

2. Import the following items:

    ```rust,ignore
    use amethyst::{
        assets::{PrefabData, ProgressCounter},
        derive::PrefabData,
        ecs::Entity,
        Error,
    };
    use serde::{Deserialize, Serialize};
    ```

3. Define the aggregate prefab data type.

    In these examples, `Named`, `Position`, and `Weapon` all derive [`PrefabData`].

    * *Superset* prefab data:

        ```rust,edition2018,no_run,noplaypen
        # extern crate amethyst;
        # extern crate derivative;
        # extern crate serde;
        # extern crate specs_derive;
        #
        # use amethyst::{
        #     assets::{PrefabData, ProgressCounter},
        #     core::Named,
        #     derive::PrefabData,
        #     ecs::{
        #         storage::{DenseVecStorage, VecStorage},
        #         Component, Entity, WriteStorage,
        #     },
        #     prelude::*,
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
        # #[derive(Clone, Copy, Component, Debug, Derivative, Deserialize, Serialize, PrefabData)]
        # #[derivative(Default)]
        # #[prefab(Component)]
        # #[storage(VecStorage)]
        # pub enum Weapon {
        #     #[derivative(Default)]
        #     Axe,
        #     Sword,
        # }
        #
        /// All fields implement `PrefabData`, and are wrapped in `Option<_>`.
        ///
        /// **Note:** If a field is not specified in the prefab, it will default
        /// to `None`.
        #[derive(Debug, Default, Deserialize, Serialize, PrefabData)]
        #[serde(default)]
        #[serde(deny_unknown_fields)]
        pub struct CustomPrefabData {
            player: Option<Named>,
            weapon: Option<Weapon>,
            position: Option<Position>,
        }
        ```

    * *Object* prefab data:

        ```rust,edition2018,no_run,noplaypen
        # extern crate amethyst;
        # extern crate derivative;
        # extern crate serde;
        # extern crate specs_derive;
        #
        # use amethyst::{
        #     assets::{PrefabData, ProgressCounter},
        #     core::Named,
        #     derive::PrefabData,
        #     ecs::{
        #         storage::DenseVecStorage,
        #         Component, Entity, WriteStorage,
        #     },
        #     prelude::*,
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
        /// **Note:** All fields must be specified in the prefab. If a field is
        /// not specified, then the prefab will fail to load.
        #[derive(Deserialize, Serialize, PrefabData)]
        #[serde(deny_unknown_fields)]
        pub struct Player {
            player: Named,
            position: Position,
        }
        ```

    The [`PrefabData`][api_pf_derive] derive implements the [`PrefabData`] trait for the type. The generated code will handle invoking the appropriate [`PrefabData`] methods when loading and attaching components to an entity. **Note:** This differs from the simple component [`PrefabData`] derive implementation &ndash; there is no `#[prefab(Component)]` attribute.

    The [`#[serde(default)]`] attribute allows fields to not be specified in the prefab, and the fields' default value will be used. If this attribute is not present, then all fields must be specified in the prefab.

    Finally, the [`#[serde(deny_unknown_fields)]`] ensures that deserialization produces an error if it encounters an unknown field. This will help expose mistakes in the prefab file, such as when there is a typo.

4. Now the type can be used in a prefab.

    * *Superset* prefab data:

        ```rust,ignore
        #![enable(implicit_some)]
        Prefab(
            entities: [
                // Player
                PrefabEntity(
                    data: CustomPrefabData(
                        player: Named(name: "Zero"),
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

    * *Object* prefab data:

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

To see this in a complete example, run the [`prefab_custom` example][repo_prefab_custom] or the [`prefab_multi` example][repo_prefab_multi] from the Amethyst repository:

```bash
cargo run --example prefab_custom # superset prefab
cargo run --example prefab_multi # object prefab
```

[`#[serde(default)]`]: https://serde.rs/container-attrs.html#default
[`#[serde(deny_unknown_fields)]`]: https://serde.rs/container-attrs.html#deny_unknown_fields
[`Component`]: https://docs-src.amethyst.rs/stable/specs/trait.Component.html
[`Prefab`]: https://docs-src.amethyst.rs/stable/amethyst_assets/struct.Prefab.html
[`PrefabData`]: https://docs-src.amethyst.rs/stable/amethyst_assets/trait.PrefabData.html#impl-PrefabData%3C%27a%3E
[api_pf_derive]: https://docs-src.amethyst.rs/stable/amethyst_derive/derive.PrefabData.html
[bk_prefab_prelude]: how_to_define_prefabs_prelude.html
[repo_prefab_custom]: https://github.com/amethyst/amethyst/tree/master/examples/prefab_custom
[repo_prefab_multi]: https://github.com/amethyst/amethyst/tree/master/examples/prefab_multi
