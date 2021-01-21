# How to Define Prefabs: Prelude

This page is **not** a guide, but since prefabs are extremely complicated, this is a dedicated page to help you choose *which guide* to use.

If you are looking for a guide for how to define prefab data that combines the components of multiple existing prefabs, please see [How to Define Prefabs: Aggregate][aggregate].

If you are looking for a guide to define prefab data for a `Component`, first we need to figure out its type based on its serialized representation. The following table summarizes the types, and links to the relevant guide. For additional detail, refer to the code snippets below the table.

| Component     | Serialized representation             | Example(s)            | Prefab Data                     | Guide          |
| ------------- | ------------------------------------- | --------------------- | ------------------------------- | -------------- |
| `YourType`    | `Self` – `YourType`                   | `Position`            | `Position`                      | [Simple]       |
| `YourType`    | Multiple – `V1(..)`, `V2(..)`         | [`Camera`]            | [`CameraPrefab`]                | [Adapter]      |
| `YourType`    | Subset of `YourType`                  | [`AudioListener`]     | [`AudioPrefab`]                 | [Asset]        |
| `Handle<A>`   | Loaded from `A::Data`                 | [`Mesh`], [`Texture`] | [`MeshData`], [`TexturePrefab`] | [Asset]        |
| `ManyHandles` | Data that component stores handles of | [`Material`]          | [`MaterialPrefab`]              | [Multi-Handle] |

### Serialized Representation

- **`Self`**

  This is where the `Component` type itself is completely serializable – the data is self-contained.

  ```rust
  # extern crate serde;
  # 
  # use serde::{Deserialize, Serialize};
  # 
  #[derive(Component, Debug, Deserialize, Serialize)]
  pub struct Position(pub f32, pub f32, pub f32);
  ```

  Applicable guide: [How to Define Prefabs: Simple][simple].

- **Multiple**

  This is where are multiple ways to construct the component, and a user should be able to choose which one to use.

  ```rust
  # extern crate serde;
  #
  # use serde::{Deserialize, Serialize};
  #
  # #[derive(Component, Debug, Deserialize, Serialize /* .. */)]
  # pub struct Position {
  #     pub x: f32,
  #     pub y: f32,
  #     pub z: f32,
  # };
  #
  impl From<(i32, i32, i32)> for Position {
      fn from((x, y, z): (i32, i32, i32)) -> Position {
          Position {
              x: x as f32,
              y: y as f32,
              z: z as f32,
          }
      }
  }

  impl From<(f32, f32, f32)> for Position {
      fn from((x, y, z): (f32, f32, f32)) -> Position {
          Position { x, y, z }
      }
  }
  ```

  Applicable guide: [How to Define Prefabs: Adapter][adapter].

- **Component Subset**

  This is where most of the component is serializable, but there is also data that is only accessible at runtime, such as a device ID or an asset handle.

  ```rust
  # use amethyst_audio::output::Output;
  # use amethyst_core::{
  #   ecs::{storage::HashMapStorage, Component},
  #   math::Point3,
  # };
  # 
  #[derive(Debug, Component)]
  # #[storage(HashMapStorage)]
  pub struct AudioListener {
      /// Output used by this listener to emit sounds to
      pub output: Output, // <--- NOTE: Only available at runtime
      // ..
  #   /// Position of the left ear relative to the global transform on this entity.
  #   pub left_ear: Point3<f32>,
  #   /// Position of the right ear relative to the global transform on this entity.
  #   pub right_ear: Point3<f32>,
  }
  ```

  Applicable guide: [How to Define Prefabs: Asset][asset].

- **Asset**

  When using `Handle<A>` as a component, `A` must `impl Asset`, and therefore `A::Data` must be serializable.

  This is where you want to load `A` as part of a prefab.

  Applicable guide: [How to Define Prefabs: Asset][asset].

- **Multi-Handle**

  This is where the `Component` itself stores `Handle<_>`s.

  ```rust
  # use amethyst::{assets::Handle, renderer::Texture};
  # 
  /// Material struct.
  #[derive(Clone, PartialEq)]
  pub struct Material {
      /// Diffuse map.
      pub albedo: Handle<Texture>,
      /// Emission map.
      pub emission: Handle<Texture>,
      // ..
  }
  ```

  Applicable guide: [How to Define Prefabs: Multi-Handle][multi-handle].

[adapter]: how_to_define_prefabs_adapter.html
[aggregate]: how_to_define_prefabs_aggregate.html
[asset]: how_to_define_prefabs_asset.html
[multi-handle]: how_to_define_prefabs_multi_handle.html
[simple]: how_to_define_prefabs_simple.html
[`audiolistener`]: https://docs.amethyst.rs/master/amethyst_audio/struct.AudioListener.html
[`audioprefab`]: https://docs.amethyst.rs/master/amethyst_audio/struct.AudioPrefab.html
[`cameraprefab`]: https://docs.amethyst.rs/master/amethyst_rendy/camera/enum.CameraPrefab.html
[`camera`]: https://docs.amethyst.rs/master/amethyst_rendy/struct.Camera.html
[`materialprefab`]: https://docs.amethyst.rs/master/amethyst_rendy/formats/mtl/struct.MaterialPrefab.html
[`material`]: https://docs.amethyst.rs/master/amethyst_rendy/struct.Material.html
[`meshdata`]: https://docs.amethyst.rs/master/amethyst_rendy/types/struct.MeshData.html
[`mesh`]: https://docs.amethyst.rs/master/amethyst_rendy/rendy/mesh/struct.Mesh.html
[`textureprefab`]: https://docs.amethyst.rs/master/amethyst_rendy/formats/texture/enum.TexturePrefab.html
[`texture`]: https://docs.amethyst.rs/master/amethyst_rendy/rendy/texture/struct.Texture.html
