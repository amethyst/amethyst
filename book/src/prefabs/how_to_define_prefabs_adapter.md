# How to Define Prefabs: Adapter

This guide explains how to define a [`PrefabData`] for a [`Component`] using an intermediate type called an adapter. This pattern is used when there are multiple ways to serialize / construct the [`Component`]:

```rust
# extern crate serde;
# 
# use serde::{Deserialize, Serialize};
# 
# #[derive(Component, Debug, Deserialize, Serialize)]
# pub struct Position(pub f32, pub f32, pub f32);
# 
impl From<(i32, i32, i32)> for Position {
    fn from((x, y, z): (i32, i32, i32)) -> Position {
        Position(x as f32, y as f32, z as f32)
    }
}

impl From<(f32, f32, f32)> for Position {
    fn from((x, y, z): (f32, f32, f32)) -> Position {
        Position(x, y, z)
    }
}
```

If you are attempting to adapt a more complex type, please choose the appropriate guide from the [available guides][bk_prefab_prelude].

## Steps

1. Ensure your crate has the following dependencies:

   ```toml
   [dependencies]
   amethyst = ".." # Minimum version 0.10
   serde = { version = "1", features = ["derive"] }
   ```

1. Define the adapter prefab data type.

   Create a (de)serializable enum type with a variant for each representation. The following is an example of an adapter type for the `Position` component, which allows either `i32` or `f32` values to be specified in the prefab:

   ```rust
   # extern crate serde;
   # 
   use amethyst::{
       assets::{PrefabData, ProgressCounter},
       ecs::Entity,
       Error,
   };
   use serde::{Deserialize, Serialize};

   #[derive(Clone, Copy, Deserialize, PartialEq, Serialize)]
   #[serde(deny_unknown_fields)]
   pub enum PositionPrefab {
       Pos3f { x: f32, y: f32, z: f32 },
       Pos3i { x: i32, y: i32, z: i32 },
   }
   ```

   The [`#[serde(deny_unknown_fields)]`][ser_unk] ensures that deserialization produces an error if it encounters an unknown field. This will help expose mistakes in the prefab file, such as when there is a typo.

   **Note:** You may already have a type that captures the multiple representations. For example, for the [`Camera`] component, the [`CameraPrefab`] enum captures the different representations:

   ```rust
   # extern crate serde;
   # 
   # use serde::{Deserialize, Serialize};
   # 
   #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
   pub enum CameraPrefab {
       Orthographic {
           left: f32,
           right: f32,
           bottom: f32,
           top: f32,
           znear: f32,
           zfar: f32,
       },
       Perspective {
           aspect: f32,
           fovy: f32,
           znear: f32,
       },
   }
   ```

1. Implement the [`PrefabData`] trait for the adapter type.

   ```rust
   # extern crate serde;
   # 
   # use amethyst::{
   #   assets::{PrefabData, ProgressCounter},
   #   ecs::{Entity},
   #   Error,
   # };
   # use serde::{Deserialize, Serialize};
   # 
   # #[derive(Component, Debug, Deserialize, Serialize)]
   # pub struct Position(pub f32, pub f32, pub f32);
   # 
   # impl From<(i32, i32, i32)> for Position {
   #   fn from((x, y, z): (i32, i32, i32)) -> Position {
   #       Position(x as f32, y as f32, z as f32)
   #   }
   # }
   # 
   # impl From<(f32, f32, f32)> for Position {
   #   fn from((x, y, z): (f32, f32, f32)) -> Position {
   #       Position(x, y, z)
   #   }
   # }
   # 
   # #[derive(Clone, Copy, Deserialize, PartialEq, Serialize)]
   # #[serde(deny_unknown_fields)]
   # pub enum PositionPrefab {
   #   Pos3f { x: f32, y: f32, z: f32 },
   #   Pos3i { x: i32, y: i32, z: i32 },
   # }
   # 
   impl<'a> PrefabData<'a> for PositionPrefab {
       // To attach the `Position` to the constructed entity,
       // we write to the `Position` component storage.
   ```

.write\_component::<Position>()

```
   // This associated type is not used in this pattern,
   // so the empty tuple is specified.
   type Result = ();

   fn add_to_entity(
       &self,
       entity: Entity,
       positions: &mut Self::SystemData,
       _entities: &[Entity],
       _children: &[Entity],
   ) -> Result<(), Error> {
       let position = match *self {
           PositionPrefab::Pos3f { x, y, z } => (x, y, z).into(),
           PositionPrefab::Pos3i { x, y, z } => (x, y, z).into(),
       };
       positions.insert(entity, position).map(|_| ())?;
       Ok(())
   }
```

}

````

1. Now the adapter type can be used in a prefab to attach the component to the entity.

```rust
#![enable(implicit_some)]
Prefab(
    entities: [
        PrefabEntity(
            data: Pos3f(x: 1.0, y: 2.0, z: 3.0),
        ),
        PrefabEntity(
            data: Pos3i(x: 4, y: 5, z: 6),
        ),
    ],
)
````

To see this in a complete example, run the [`prefab_adapter` example][repo_prefab_adapter] from the Amethyst repository:

```bash
cargo run -p prefab_adapter
```

[bk_prefab_prelude]: how_to_define_prefabs_prelude.html
[repo_prefab_adapter]: https://github.com/amethyst/amethyst/tree/master/examples/prefab_adapter
[ser_unk]: https://serde.rs/container-attrs.html#deny_unknown_fields
[`cameraprefab`]: https://docs.amethyst.rs/master/amethyst_rendy/camera/enum.CameraPrefab.html
[`camera`]: https://docs.amethyst.rs/master/amethyst_rendy/camera/struct.Camera.html
[`component`]: https://docs.rs/specs/~0.16/specs/trait.Component.html
[`prefabdata`]: https://docs.amethyst.rs/master/amethyst_assets/trait.PrefabData.html
