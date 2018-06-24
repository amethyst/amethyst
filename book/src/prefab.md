# Prefabs

A `Prefab` in `Amethyst` is at the core a simple list of future entities, where each entry in 
the list consists of two pieces of optional data:

* a parent index that refers to a different entry in the list
* a data collection implementing the trait `PrefabData`

To instantiate a `Prefab`, we put a `Handle<Prefab<T>>` on an `Entity`. The `Entity` we put 
the `Handle` on is referred to as the main `Entity`, and the first entry in the list inside a 
`Prefab` refers to this `Entity`. All other entries in the list will spawn a new `Entity` on 
instantiation. 

NOTE: This means that we currently cannot target multiple existing entities from a single `Prefab`.
This restriction is likely to be removed in the future.

The lifetime of a `Prefab` can roughy be divided into three distinct parts:

**Loading**

This is the same as for all assets in `Amethyst`, the user initiates a load using `Loader`, a 
`Source` and a `Format`. The `Format` returns a `Prefab`, and the user is handed a `Handle<Prefab<T>>`,
for some `T` that implements `PrefabData`.

**Sub asset loading**

A `PrefabData` implementation could refer to other assets that need to be loaded asynchronously, and
we don't want the user get a `Complete` notification on their `Progress` before everything has been 
loaded.

Because of this, once the `Format` have loaded the `Prefab` from the `Source`, and a `PrefabLoaderSystem`
runs `process` on the `AssetStorage`, the system will invoke the `trigger_sub_loading` function on the
`PrefabData` implementation. If any asset loads are triggered during this, they must adhere to the following
rules:

* the given `ProgressCounter` must be used as a parameter to the load function on `Loader`, so load tracking
works correctly
* the function must return `Ok(true)` (unless an `Error` occurred)

Note that during this phase the `PrefabData` is mutable, which means it can morph inside the `Prefab`. An
example of this is the `AssetPrefab`, which will morph into `AssetPrefab::Handle`.

Once all sub asset loading is finished, which the `PrefabLoaderSystem` will track using the `ProgressCounter`,
a `Complete` signal will be sent upwards.

**Prefab instantiation**

This stage happens after the `Prefab` has been fully loaded and `Complete` has been signaled, and the 
`Handle<Prefab<T>>` is put on an `Entity`. At this point we know that all internal data has been loaded, 
and all sub assets have been processed. The `PrefabLoaderSystem` will then walk through the `Prefab` data 
immutably and create a new `Entity` for all but the first entry in the list, and then for each instance 
of `PrefabData` call the `load_prefab` function.

## `PrefabData`

Ok, so what would a simple implementation of `PrefabData` look like?

Let's take a look at the implementation for `Transform`, which is a core concept in `Amethyst`:

```rust,ignore
impl<'a> PrefabData<'a> for Transform {
    type SystemData = (
        WriteStorage<'a, Transform>,
        WriteStorage<'a, GlobalTransform>,
    );
    type Result = ();

    fn load_prefab(
        &self,
        entity: Entity,
        storages: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), SpecsError> {
        storages.1.insert(entity, GlobalTransform::default())?;
        storages.0.insert(entity, self.clone()).map(|_| ())
    }
}
```

First, we specify a `SystemData` type, this is the data required from `World` in order to load and
instantiate this `PrefabData`. Here we want to write to both `Transform` and `GlobalTransform`, 
because `Transform` won't work without a companion `GlobalTransform`.

Second, we specify what result the `load_prefab` function returns. In our case this is unit `()`, for 
other implementations it could return a `Handle` etc. For an example of this, look at the `TexturePrefab`
in the renderer crate. 

Next we defined the `load_prefab` function, which is used to actually instantiate data. In our case here,
we insert a default `GlobalTransform` and the local `Transform` data on the referenced `Entity`. In this
scenario we aren't using the third parameter to the function. This parameter contains a list of all `Entity`s
affected by the `Prefab`, the first entry in the list will be the main `Entity`, and the rest will be the 
`Entity`s that were created for all the entries in the data list inside the `Prefab`.

Last of all, we can see that this does not implement `trigger_sub_loading`, which is because there
are no secondary assets to load from `Source` here.

Let's look at a slightly more complex implementation, the `AssetPrefab`. This `PrefabData` is used to
load extra `Asset`s as part of a `Prefab`:

```rust,ignore
#[derive(Deserialize, Serialize)]
pub enum AssetPrefab<A, F>
where
    A: Asset,
    F: Format<A>,
{
    /// From existing handle
    #[serde(skip)]
    Handle(Handle<A>),

    /// From file, (name, format, format options)
    File(String, F, F::Options),
}

impl<'a, A, F> PrefabData<'a> for AssetPrefab<A, F>
where
    A: Asset,
    F: Format<A> + Clone,
    F::Options: Clone,
{
    type SystemData = (
        ReadExpect<'a, Loader>,
        WriteStorage<'a, Handle<A>>,
        Read<'a, AssetStorage<A>>,
    );

    type Result = Handle<A>;

    fn load_prefab(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<Handle<A>, SpecsError> {
        let handle = match *self {
            AssetPrefab::Handle(ref handle) => handle.clone(),
            AssetPrefab::File(ref name, ref format, ref options) => system_data.0.load(
                name.as_ref(),
                format.clone(),
                options.clone(),
                (),
                &system_data.2,
            ),
        };
        system_data.1.insert(entity, handle.clone()).map(|_| handle)
    }

    fn trigger_sub_loading(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, SpecsError> {
        let handle = match *self {
            AssetPrefab::File(ref name, ref format, ref options) => Some(system_data.0.load(
                name.as_ref(),
                format.clone(),
                options.clone(),
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
The first difference is that the `load_prefab` function now return a `Handle<A>`.
The second difference is that `trigger_sub_loading` is implemented, this is because we load 
a sub asset. The `trigger_sub_loading` function here will do the actual loading, and morph the
internal representation to the `AssetPrefab::Handle` variant, so when `load_prefab` runs later
it will straight up use the internally stored `Handle`.

### Special `PrefabData` implementations

There are a few special blanket implementations provided by the asset system:

* `Option<T>` for all `T: PrefabData`.
* Tuples of types that implemented `PrefabData`, up to a size of 20.

## Working with `Prefab`s

So now we know how the `Prefab` system works on the inside, but how do we use it?

From the point of the user, there are a few parts to using a `Prefab`: 

* Loading it, using `Loader` + `AssetStorage`, or using the helper `PrefabLoader`, which is a
 simple wrapper around the former. For this to work we need a `Format` that returns `Prefab`s.
* Managing the returned `Handle<Prefab<T>>`.
* Waiting for the `Prefab` to be fully loaded, using `Progress`.
* Requesting instantiation by placing the `Handle<Prefab<T>>` on an `Entity` in the `World`.

## `Prefab` formats

There are a few provided formats that create `Prefab`s, some with very specific `PrefabData`, and
 one that is generic:
 
* `RonFormat` - this format can be used to load `Prefab`s in `ron` format with any `PrefabData`
 that also implements `serde::Deserialize`.
* `GltfSceneFormat` - used to load `Gltf` files
* `UiFormat` - used to load UI components in a specialised DSL format.

For an example of a `Prefab` in `ron` format, look at `examples/assets/prefab/example.ron`. The
`PrefabData` for this is:
 
```rust,ignore
(
    Option<GraphicsPrefab<ObjFormat, TextureFormat>>,
    Option<Transform>,
    Option<Light>,
    Option<CameraPrefab>,
)
```

For a more advanced example, and also a custom `PrefabData` implementation, look at the `gltf` example 
and `examples/assets/prefab/puffy_scene.ron`.
