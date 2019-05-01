# How to Define Prefabs: Prelude

This page is **not** a guide, but since prefabs are extremely complicated, this is a dedicated page to help you choose *which guide* to use.

If you are looking for a guide for how to define prefab data that combines the components of multiple existing prefabs, please see [How to Define Prefabs: Aggregate][Aggregate].

If you are looking for a guide to define prefab data for a `Component`, first we need to figure out its type based on its serialized representation. The following table summarizes the types, and links to the relevant guide. For additional detail, refer to the code snippets below the table.

Component     | Serialized representation             | Example(s)            | Prefab Data        | Guide
------------- | ------------------------------------- | --------------------- | ------------------ | ---
`YourType`    | `Self` &ndash; `YourType`             | `Position`            | `Position`         | [Simple]
`YourType`    | Multiple &ndash; `V1(..)`, `V2(..)`   | [`Camera`]            | [`CameraPrefab`]   | [Adapter]
`YourType`    | Subset of `YourType`                  | [`AudioListener`]     | [`AudioPrefab`]    | [Asset]
`Handle<A>`   | Loaded from `A::Data`                 | [`Mesh`], [`Texture`] | [`MeshData`], [`TexturePrefab`] | [Asset]
`ManyHandles` | Data that component stores handles of | [`Material`]          | [`MaterialPrefab`] | [Multi-Handle]

### Serialized Representation

* **`Self`**

    This is where the `Component` type itself is completely serializable &ndash; the data is self-contained.

    ```rust,edition2018,no_run,noplaypen
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

    Applicable guide: [How to Define Prefabs: Simple][Simple].

* **Multiple**

    This is where are multiple ways to construct the component, and a user should be able to choose which one to use.

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    # extern crate serde;
    # extern crate specs_derive;
    #
    # use amethyst::ecs::{storage::DenseVecStorage, Component};
    # use serde::{Deserialize, Serialize};
    # use specs_derive::Component;
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

    Applicable guide: [How to Define Prefabs: Adapter][Adapter].

* **Component Subset**

    This is where most of the component is serializable, but there is also data that is only accessible at runtime, such as a device ID or an asset handle.

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst_audio;
    # extern crate amethyst_core;
    # extern crate specs_derive;
    #
    # use amethyst_audio::output::Output;
    # use amethyst_core::{
    #     math::Point3,
    #     ecs::{prelude::Component, storage::HashMapStorage},
    # };
    # use specs_derive::Component;
    #
    #[derive(Debug, Component)]
    # #[storage(HashMapStorage)]
    pub struct AudioListener {
        /// Output used by this listener to emit sounds to
        pub output: Output, // <--- NOTE: Only available at runtime
        // ..
    #     /// Position of the left ear relative to the global transform on this entity.
    #     pub left_ear: Point3<f32>,
    #     /// Position of the right ear relative to the global transform on this entity.
    #     pub right_ear: Point3<f32>,
    }
    ```

    Applicable guide: [How to Define Prefabs: Asset][Asset].

* **Asset**

    When using `Handle<A>` as a component, `A` must `impl Asset`, and therefore `A::Data` must be serializable.

    This is where you want to load `A` as part of a prefab.

    Applicable guide: [How to Define Prefabs: Asset][Asset].

* **Multi-Handle**

    This is where the `Component` itself stores `Handle<_>`s.

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    #
    # use amethyst::{
    #     ecs::{storage::DenseVecStorage, Component},
    #     renderer::{TextureHandle, TextureOffset},
    # };
    #
    /// Material struct.
    #[derive(Clone, PartialEq)]
    pub struct Material {
        /// Diffuse map.
        pub albedo: TextureHandle,
        /// Diffuse texture offset
        pub albedo_offset: TextureOffset,
        /// Emission map.
        pub emission: TextureHandle,
        /// Emission texture offset
        pub emission_offset: TextureOffset,
        // ..
    }

    impl Component for Material {
        type Storage = DenseVecStorage<Self>;
    }
    ```

    Applicable guide: [How to Define Prefabs: Multi-Handle][Multi-Handle].

[`AudioListener`]: https://www.amethyst.rs/doc/latest/doc/amethyst_audio/struct.AudioListener.html
[`AudioPrefab`]: https://www.amethyst.rs/doc/latest/doc/amethyst_audio/struct.AudioPrefab.html
[`Camera`]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/struct.Camera.html
[`CameraPrefab`]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/enum.CameraPrefab.html
[`Material`]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/struct.Material.html
[`MaterialPrefab`]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/struct.MaterialPrefab.html
[`Mesh`]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/struct.Mesh.html
[`MeshData`]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/enum.MeshData.html
[`Texture`]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/struct.Texture.html
[`TexturePrefab`]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/enum.TexturePrefab.html
[Adapter]: how_to_define_prefabs_adapter.html
[Asset]: how_to_define_prefabs_asset.html
[Aggregate]: how_to_define_prefabs_aggregate.html
[Multi-Handle]: how_to_define_prefabs_multi_handle.html
[Simple]: how_to_define_prefabs_simple.html
