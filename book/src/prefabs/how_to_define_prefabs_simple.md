# How to Define Prefabs: Simple

This guide explains how to enable a [`Component`] to be used in a [`Prefab`]. This can be applied where the [`Component`] type itself is completely serializable – the data is self-contained:

```rust
# extern crate serde;
# 
# use serde::{Deserialize, Serialize};
# 
#[derive(Component, Debug, Deserialize, Serialize)]
pub struct Position(pub f32, pub f32, pub f32);
```

If you are attempting to adapt a more complex type, please choose the appropriate guide from the [available guides][bk_prefab_prelude].

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

1. Add the following attributes on your type:

   ```rust
   #[derive(Deserialize, Serialize, PrefabData)]
   #[prefab(Component)]
   #[serde(default)] // <--- optional
   #[serde(deny_unknown_fields)]
   ```

   Example:

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
   #[serde(deny_unknown_fields)]
   pub struct Position(pub f32, pub f32, pub f32);
   ```

   The [`PrefabData`] [api\_pf\_derive][api_pf_derive] derive implements the [`PrefabData`] trait for the type. The `#[prefab(Component)]` attribute informs the [`PrefabData`] derive that this type is a [`Component`], as opposed to being composed of fields which implement `[PrefabData]`.

   The [`#[serde(default)]`][ser_def] attribute allows fields to not be specified in the prefab, and the fields' default value will be used. If this attribute is not present, then all fields must be specified in the prefab.

   Finally, the [`#[serde(deny_unknown_fields)]`][ser_unk] ensures that deserialization produces an error if it encounters an unknown field. This will help expose mistakes in the prefab file, such as when there is a typo.

1. Now the type can be used in a prefab:

   ```rust
   #![enable(implicit_some)]
   Prefab(
       entities: [
           PrefabEntity(
               data: Position(1.0, 2.0, 3.0),
           ),
       ],
   )
   ```

To see this in a complete example, run the [`prefab` example] from the Amethyst repository:

```bash
cargo run -p prefab
```

[api_pf_derive]: https://docs.amethyst.rs/master/amethyst_derive/derive.PrefabData.html
[bk_prefab_prelude]: ./how_to_define_prefabs_prelude.html
[ser_def]: https://serde.rs/container-attrs.html#default
[ser_unk]: https://serde.rs/container-attrs.html#deny_unknown_fields
[`component`]: https://docs.rs/specs/~0.16/specs/trait.Component.html
[`prefabdata`]: https://docs.amethyst.rs/master/amethyst_assets/trait.PrefabData.html#impl-PrefabData
[`prefab`]: https://docs.amethyst.rs/master/amethyst_assets/struct.Prefab.html
[`prefab` example]: https://github.com/amethyst/amethyst/tree/master/examples/prefab
