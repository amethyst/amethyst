# Prefabs Technical Explanation

A `Prefab` in Amethyst is at the core a simple list of future entities, where each entry in
the list consists of two pieces of optional data:

- a parent index that refers to a different entry in the list
- a data collection implementing the trait `PrefabData`

To instantiate a `Prefab`, we put a `Handle<Prefab<T>>` on an `Entity`. The `Entity` we put
the `Handle` on is referred to as the main `Entity`, and the first entry in the list inside a
`Prefab` refers to this `Entity`. All other entries in the list will spawn a new `Entity` on
instantiation.

NOTE: This means that we currently cannot target multiple existing entities from a single `Prefab`.
This restriction is likely to be removed in the future.

The lifetime of a `Prefab` can roughly be divided into three distinct parts:

**Loading**

This is the same as for all assets in Amethyst, the user initiates a load using `Loader`, a
`Source` and a `Format`. The `Format` returns a `Prefab`, and the user is handed a `Handle<Prefab<T>>`,
for some `T` that implements `PrefabData`.

**Sub asset loading**

A `PrefabData` implementation could refer to other assets that need to be loaded asynchronously, and
we don't want the user get a `Complete` notification on their `Progress` before everything has been
loaded.

Because of this, once the `Format` have loaded the `Prefab` from the `Source`, and a `PrefabLoaderSystem`
runs `process` on the `AssetStorage`, the system will invoke the `load_sub_assets` function on the
`PrefabData` implementation. If any asset loads are triggered during this, they must adhere to the following
rules:

- the given `ProgressCounter` must be used as a parameter to the load function on `Loader`, so load tracking
  works correctly
- the function must return `Ok(true)` (unless an `Error` occurred)

Note that during this phase the `PrefabData` is mutable, which means it can morph inside the `Prefab`. An
example of this is the `AssetPrefab`, which will morph into `AssetPrefab::Handle`.

Once all sub asset loading is finished, which the `PrefabLoaderSystem` will track using the `ProgressCounter`,
a `Complete` signal will be sent upwards.

**Prefab instantiation**

This stage happens after the `Prefab` has been fully loaded and `Complete` has been signaled, and the
`Handle<Prefab<T>>` is put on an `Entity`. At this point we know that all internal data has been loaded,
and all sub assets have been processed. The `PrefabLoaderSystem` will then walk through the `Prefab` data
immutably and create a new `Entity` for all but the first entry in the list, and then for each instance
of `PrefabData` call the `add_to_entity` function.

Note that for prefabs that reference other prefabs, to make instantiation be performed inside a single frame,
lower level `PrefabLoaderSystem`s need to depend on the higher level ones. To see how this works out check the gltf
example, where we have a scene prefab, and the gltf loader (which use the prefab system internally).

## `PrefabData`

Ok, so what would a simple implementation of `PrefabData` look like?

Let's take a look at the implementation for `Transform`, which is a core concept in Amethyst:

```rust
# use amethyst::assets::PrefabData;
# use amethyst::ecs::{Entity};
# use amethyst::Error;
# 
# // We declare that struct for the sake of automated testing.
# #[derive(Default, Clone)]
# struct Transform;
# 
impl<'a> PrefabData<'a> for Transform {
.write_component::<Transform>()
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        storage: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        storage
            .insert(entity, self.clone())
            .map(|_| ())
            .map_err(Into::into)
    }
}
```

First, we specify a `SystemData` type, this is the data required from `World` in order to load and
instantiate this `PrefabData`. Here we only need to write to `Transform`.

Second, we specify what result the `add_to_entity` function returns. In our case this is unit `()`, for
other implementations it could return a `Handle` etc. For an example of this, look at the `TexturePrefab`
in the renderer crate.

Next, we define the `add_to_entity` function, which is used to actually instantiate data. In our case here,
we insert the local `Transform` data on the referenced `Entity`. In this scenario we aren't using the third
parameter to the function. This parameter contains a list of all entities affected by the `Prefab`, the first
entry in the list will be the main `Entity`, and the rest will be the entities that were created for all the
entries in the data list inside the `Prefab`.

Last of all, we can see that this does not implement `load_sub_assets`, which is because there
are no secondary assets to load from `Source` here.

Let's look at a slightly more complex implementation, the `AssetPrefab`. This `PrefabData` is used to
load extra `Asset`s as part of a `Prefab`:

```rust
# use amethyst::assets::PrefabData;
# use amethyst::assets::{
#   Asset, AssetStorage, DefaultLoader, Format, Handle, Loader, ProgressCounter,
# };
# use amethyst::ecs::{Entity};
# use amethyst::Error;
# 
#[derive(Deserialize, Serialize)]
pub enum AssetPrefab<A, F>
where
    A: Asset,
    F: Format<A::Data>,
{
    /// From existing handle
    #[serde(skip)]
    Handle(Handle<A>),

    /// From file, (name, format, format options)
    File(String, F),
}

impl<'a, A, F> PrefabData<'a> for AssetPrefab<A, F>
where
    A: Asset,
    F: Format<A::Data> + Clone,
{
    type SystemData = (
        ReadExpect<'a, Loader>,
        .write_component::<Handle<A>>()
        .read_resource::<AssetStorage<A>>(),
    );

    type Result = Handle<A>;

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<Handle<A>, Error> {
        let handle = match *self {
            AssetPrefab::Handle(ref handle) => handle.clone(),
            AssetPrefab::File(ref name, ref format) => {
                system_data
                    .0
                    .load(name.as_str(), format.clone(), (), &system_data.2)
            }
        };
        Ok(system_data.1.insert(entity, handle.clone())?.unwrap())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let handle = match *self {
            AssetPrefab::File(ref name, ref format) => Some(system_data.0.load(
                name.as_str(),
                format.clone(),
                progress,
                &system_data.2,
            )),
            _ => None,
        };
        if let Some(handle) = handle {
            *self = AssetPrefab::Handle(handle);
        }
        Ok(true)
    }
}
```

So, there are two main differences to this `PrefabData` compared the `Transform` example.
The first difference is that the `add_to_entity` function now return a `Handle<A>`.
The second difference is that `load_sub_assets` is implemented, this is because we load
a sub asset. The `load_sub_assets` function here will do the actual loading, and morph the
internal representation to the `AssetPrefab::Handle` variant, so when `add_to_entity` runs later
it will straight up use the internally stored `Handle`.

### Special `PrefabData` implementations

There are a few special blanket implementations provided by the asset system:

- `Option<T>` for all `T: PrefabData`.
- Tuples of types that implemented `PrefabData`, up to a size of 20.

### Deriving `PrefabData` implementations

Amethyst supplies a derive macro for creating the `PrefabData` implementation for the following scenarios:

- Single `Component`
- Aggregate `PrefabData` structs or enums which contain other `PrefabData` constructs, and optionally simple data `Component`s

In addition, deriving a `Prefab` requires that `amethyst::Error`, `amethyst::ecs::Entity` and
`amethyst:assets::{PrefabData, ProgressCounter}` are imported
and visible in the current scope. This is due to how Rust macros work.

An example of a single `Component` derive:

```rust
# use amethyst::{
#   assets::{Asset, AssetStorage, Format, Handle, Loader, PrefabData, ProgressCounter},
#   derive::PrefabData,
#   ecs::Entity,
#   Error,
# };
# 
#[derive(Clone, PrefabData)]
#[prefab(Component)]
pub struct SomeComponent {
    pub id: u64,
}
```

This will derive a `PrefabData` implementation that inserts `SomeComponent` on an `Entity` in the `World`.

Lets look at an example of an aggregate struct:

```rust
# use amethyst::assets::{
#   Asset, AssetPrefab, AssetStorage, DefaultLoader, Format, Handle, Loader, PrefabData,
#   ProgressCounter,
# };
# use amethyst::core::Transform;
# use amethyst::ecs::Entity;
# use amethyst::renderer::{formats::mesh::ObjFormat, Mesh};
# use amethyst::Error;

#[derive(PrefabData)]
pub struct MyScenePrefab {
    mesh: AssetPrefab<Mesh, ObjFormat>,
    transform: Transform,
}
```

This can now be used to create `Prefab`s with `Transform` and `Mesh` on entities.

One last example that also adds a custom pure data `Component` into the aggregate `PrefabData`:

```rust
# use amethyst::assets::{
#   Asset, AssetPrefab, AssetStorage, DefaultLoader, Format, Handle, Loader, PrefabData,
#   ProgressCounter,
# };
# use amethyst::core::Transform;
# use amethyst::ecs::Entity;
# use amethyst::renderer::{formats::mesh::ObjFormat, Mesh};
# use amethyst::Error;

#[derive(PrefabData)]
pub struct MyScenePrefab {
    mesh: AssetPrefab<Mesh, ObjFormat>,
    transform: Transform,

    #[prefab(Component)]
    some: SomeComponent,
}

#[derive(Clone)]
pub struct SomeComponent {
    pub id: u64,
}
```

You might notice here that `SomeComponent` has no `PrefabData` derive on its own, it is simply
used directly in the aggregate `PrefabData`, and annotated so the derive knows to do a simple
`WriteStorage` insert.

## Working with `Prefab`s

So now we know how the `Prefab` system works on the inside, but how do we use it?

From the point of the user, there are a few parts to using a `Prefab`:

- Loading it, using `Loader` + `AssetStorage`, or using the helper `PrefabLoader`, which is a
  simple wrapper around the former. For this to work we need a `Format` that returns `Prefab`s.
- Managing the returned `Handle<Prefab<T>>`.
- Waiting for the `Prefab` to be fully loaded, using `Progress`.
- Requesting instantiation by placing the `Handle<Prefab<T>>` on an `Entity` in the `World`.

## `Prefab` formats

There are a few provided formats that create `Prefab`s, some with very specific `PrefabData`, and
two that are generic:

- `RonFormat` - this format can be used to load `Prefab`s in `ron` format with any `PrefabData`
  that also implements `serde::Deserialize`.
- `JsonFormat` - this format can be used to load `Prefab`s in `Json` format with any `PrefabData`
  that also implements `serde::Deserialize`. It can be enabled with the `json` feature flag.
- `GltfSceneFormat` - used to load `Gltf` files
- `UiFormat` - used to load UI components in a specialised DSL format.

For an example of a `Prefab` in `ron` format, look at `examples/assets/prefab/example.ron`. The
`PrefabData` for this is:

```rust
(
    Option<GraphicsPrefab<ObjFormat, TextureFormat>>,
    Option<Transform>,
    Option<Light>,
    Option<CameraPrefab>,
)
```

For a more advanced example, and also a custom `PrefabData` implementation, look at the `gltf` example
and `examples/assets/prefab/puffy_scene.ron`.
