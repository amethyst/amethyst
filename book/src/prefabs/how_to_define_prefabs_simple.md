# How to Define Prefabs: Simple

This guide explains how to enable a [`Component`] to be used in a [`Prefab`]. This can be applied where the [`Component`] type itself is completely serializable &ndash; the data is self-contained:

```rust,no_run,noplaypen
# extern crate amethyst;
# extern crate serde;
# extern crate specs_derive;
#
# use amethyst::ecs::{storage::DenseVecStorage, Component};
# use serde::{Deserialize, Serialize};
# use specs_derive::Component;
#
#[derive(Component, Debug, Deserialize, Serialize /* .. */)]
pub struct Position(pub f32, pub f32, pub f32);
```

If you are attempting to adapt a more complex type, please choose the appropriate guide from the [available guides][bk_prefab_prelude].

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

3. Add the following attributes on your type:

    ```rust,ignore
    #[derive(Deserialize, Serialize, PrefabData)]
    #[prefab(Component)]
    #[serde(default)] // <--- optional
    #[serde(deny_unknown_fields)]
    ```

    Example:

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    # extern crate derivative;
    # extern crate serde;
    # extern crate specs_derive;
    #
    # use amethyst::{
    #     assets::{Prefab, PrefabData},
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
    #[derive(Clone, Copy, Component, Debug, Default, Deserialize, Serialize, PrefabData)]
    #[prefab(Component)]
    #[serde(deny_unknown_fields)]
    pub struct Position(pub f32, pub f32, pub f32);
    ```

    The [`PrefabData`][api_pf_derive] derive implements the [`PrefabData`] trait for the type. The `#[prefab(Component)]` attribute informs the [`PrefabData`] derive that this type is a [`Component`], as opposed to being composed of fields which implement [`PrefabData`].

    The [`#[serde(default)]`] attribute allows fields to not be specified in the prefab, and the fields' default value will be used. If this attribute is not present, then all fields must be specified in the prefab.

    Finally, the [`#[serde(deny_unknown_fields)]`] ensures that deserialization produces an error if it encounters an unknown field. This will help expose mistakes in the prefab file, such as when there is a typo.

4. Now the type can be used in a prefab:

    ```rust,ignore
    #![enable(implicit_some)]
    Prefab(
        entities: [
            PrefabEntity(
                data: Position(1.0, 2.0, 3.0),
            ),
        ],
    )
    ```

To see this in a complete example, run the [`prefab_basic` example][repo_prefab_basic] from the Amethyst repository:

```bash
cargo run --example prefab_basic
```

[`#[serde(default)]`]: https://serde.rs/container-attrs.html#default
[`#[serde(deny_unknown_fields)]`]: https://serde.rs/container-attrs.html#deny_unknown_fields
[`Component`]: https://docs-src.amethyst.rs/stable/specs/trait.Component.html
[`Prefab`]: https://docs-src.amethyst.rs/stable/amethyst_assets/struct.Prefab.html
[`PrefabData`]: https://docs-src.amethyst.rs/stable/amethyst_assets/trait.PrefabData.html#impl-PrefabData%3C%27a%3E
[api_pf_derive]: https://docs-src.amethyst.rs/stable/amethyst_derive/derive.PrefabData.html
[bk_prefab_prelude]: how_to_define_prefabs_prelude.html
[repo_prefab_basic]: https://github.com/amethyst/amethyst/tree/master/examples/prefab_basic
