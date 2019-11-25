# How to Use Assets

This guide covers the basic usage of assets into Amethyst for existing supported formats. For a list of supported formats, please [use this search for "Format"][doc_search_format] in the API documentation, and filter by the following crates:

* amethyst_assets
* amethyst_audio
* amethyst_gltf
* amethyst_locale
* amethyst_ui

## Steps

1. Instantiate the Amethyst application with the assets directory.

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    #
    use amethyst::{
        prelude::*,
    #   ecs::{World, WorldExt},
        utils::application_root_dir,
    };
    #
    # pub struct LoadingState;
    # impl SimpleState for LoadingState {}

    fn main() -> amethyst::Result<()> {
        // Sets up the application to read assets in
        // `<app_dir>/assets`
        let app_root = application_root_dir()?;
        let assets_dir = app_root.join("assets");

        //..
    #   let world = World::new();
    #   let game_data = GameDataBuilder::default();

        let mut game = Application::new(assets_dir, LoadingState, game_data)?;
    #
    #   game.run();
    #   Ok(())
    }
    ```

2. Ensure that the [`Processor<A>` system][doc_processor_system] for asset type `A` is registered in the dispatcher.

    For asset type `A`, `Processor<A>` is a `System` that will asynchronously load `A` assets. Usually the crate that provides `A` will also register `Processor<A>` through a `SystemBundle`. Examples:

    * `FontAsset` is provided by `amethyst_ui`, `UiBundle` registers `Processor<FontAsset>`.
    * `Source` is provided by `amethyst_audio`, `AudioBundle` registers `Processor<Source>`.
    * `SpriteSheet` is not added by a bundle, so `Processor<SpriteSheet>` needs to be added
      to the builder.

3. Use the [`Loader`][doc_loader] resource to load the asset.

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    # use amethyst::{
    #     assets::{AssetStorage, Handle, Loader, ProgressCounter},
    #     ecs::{World, WorldExt},
    #     prelude::*,
    #     renderer::{formats::texture::ImageFormat, Texture},
    #     utils::application_root_dir,
    # };
    #
    pub struct LoadingState {
        /// Tracks loaded assets.
        progress_counter: ProgressCounter,
        /// Handle to the player texture.
        texture_handle: Option<Handle<Texture>>,
    }

    impl SimpleState for LoadingState {
        fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
            let loader = &data.world.read_resource::<Loader>();
            let texture_handle = loader.load(
                "player.png",
                ImageFormat::default(),
                &mut self.progress_counter,
                &data.world.read_resource::<AssetStorage<Texture>>(),
            );

            self.texture_handle = Some(texture_handle);
        }
    }
    #
    # fn main() -> amethyst::Result<()> {
    #   let app_root = application_root_dir()?;
    #   let assets_dir = app_root.join("assets");
    #
    #   let game_data = GameDataBuilder::default();
    #   let mut game = Application::new(
    #       assets_dir,
    #       LoadingState {
    #           progress_counter: ProgressCounter::new(),
    #           texture_handle: None,
    #       },
    #       game_data,
    #   )?;
    #
    #   game.run();
    #   Ok(())
    # }
    ```

4. Wait for the asset to be loaded.

    When [`loader.load(..)`][doc_load] is used to load an [`Asset`][doc_asset], the method returns immediately with a handle for the asset. The asset loading is handled asynchronously in the background, so if the handle is used to retrieve the asset, such as with [`world.read_resource::<AssetStorage<Texture>>()`][doc_read_resource][`.get(texture_handle)`][doc_asset_get], it will return `None` until the `Texture` has finished loading.

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    # use amethyst::{
    #     assets::{Handle, ProgressCounter},
    #     prelude::*,
    #     renderer::Texture,
    # };
    #
    # pub struct GameState {
    #     /// Handle to the player texture.
    #     texture_handle: Handle<Texture>,
    # }
    #
    # impl SimpleState for GameState {}
    #
    # pub struct LoadingState {
    #     /// Tracks loaded assets.
    #     progress_counter: ProgressCounter,
    #     /// Handle to the player texture.
    #     texture_handle: Option<Handle<Texture>>,
    # }
    #
    impl SimpleState for LoadingState {
        fn update(
            &mut self,
            _data: &mut StateData<'_, GameData<'_, '_>>,
        ) -> SimpleTrans {
            if self.progress_counter.is_complete() {
                Trans::Switch(Box::new(GameState {
                    texture_handle: self.texture_handle
                        .take()
                        .expect(
                            "Expected `texture_handle` to exist when \
                            `progress_counter` is complete."
                        ),
                }))
            } else {
                Trans::None
            }
        }
    }
    ```

   The asset handle can now be used:

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    # use amethyst::{
    #     assets::Handle,
    #     prelude::*,
    #     renderer::Texture,
    # };
    #
    # pub struct GameState {
    #     /// Handle to the player texture.
    #     texture_handle: Handle<Texture>,
    # }
    #
    impl SimpleState for GameState {
        fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
            // Create the player entity.
            data.world
                .create_entity()
                // Use the texture handle as a component
                .with(self.texture_handle.clone())
                .build();
        }
    }
    ```

[doc_asset]: https://docs.amethyst.rs/stable/amethyst_assets/trait.Asset.html
[doc_asset_get]: https://docs.amethyst.rs/stable/amethyst_assets/struct.AssetStorage.html#method.get
[doc_loader]: https://docs.amethyst.rs/stable/amethyst_assets/struct.Loader.html
[doc_load]: https://docs.amethyst.rs/stable/amethyst_assets/struct.Loader.html#method.load
[doc_processor_system]: https://docs.amethyst.rs/stable/amethyst_assets/struct.Processor.html
[doc_read_resource]: https://docs.amethyst.rs/stable/specs/world/struct.World.html#method.read_resource
[doc_search_format]: https://docs.amethyst.rs/stable/amethyst/?search=Format
