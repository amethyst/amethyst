# How to Use Assets

This guide covers the basic usage of assets into Amethyst for existing supported formats. For a list of supported formats, please [use this search for "Format"][doc_search_format] in the API documentation, and filter by the following crates:

- amethyst\_assets
- amethyst\_audio
- amethyst\_gltf
- amethyst\_locale
- amethyst\_ui

## Steps

1. Instantiate the Amethyst application with the assets directory.

   ```rust
   use amethyst::prelude::*;
   use amethyst::utils::application_root_dir;

   pub struct LoadingState;
   impl SimpleState for LoadingState {}

   fn main() -> amethyst::Result<()> {
       // Sets up the application to read assets in
       // `<app_dir>/assets`
       let app_root = application_root_dir()?;
       let assets_dir = app_root.join("assets");

       let game_data = DispatcherBuilder::default();

       let mut game = Application::new(assets_dir, LoadingState, game_data)?;
       //game.run();
       Ok(())
   }
   ```

1. Ensure that the [`AssetProcessorSystem<A>` system][doc_processor_system] for asset type `A` is registered in the dispatcher.
   You can use `register_asset_type!` macro for this.

   For asset type `A`, `AssetProcessorSystem<A>` is a `System` that will asynchronously load `A` assets. Usually the crate that provides `A` will also register `AssetProcessorSystem<A>` through a `SystemBundle`. Examples:

   - `FontAsset` is provided by `amethyst_ui`, `UiBundle` registers `AssetProcessorSystem<FontAsset>`.
   - `Source` is provided by `amethyst_audio`, `AudioBundle` registers `AssetProcessorSystem<Source>`.

1. Use the [`Loader`][doc_loader] resource to load the asset.

   ```rust
   # use amethyst::prelude::*;
   # use amethyst::utils::application_root_dir;
   # use amethyst::{
   #   assets::{AssetStorage, DefaultLoader, Handle, Loader, ProgressCounter},
   #   renderer::{formats::texture::ImageFormat, Texture},
   # };
   # 
   pub struct LoadingState {
       /// Handle to the player texture.
       texture_handle: Option<Handle<Texture>>,
   }

   impl SimpleState for LoadingState {
       fn on_start(&mut self, data: StateData<'_, GameData>) {
           let loader = data.resources.get::<DefaultLoader>().unwrap();
           let texture_handle = loader.load("player.png");

           self.texture_handle = Some(texture_handle);
       }
   }
   # fn main() -> amethyst::Result<()> {
   #   let app_root = application_root_dir()?;
   #   let assets_dir = app_root.join("assets");
   # 
   #   let game_data = DispatcherBuilder::default();
   #   let mut game = Application::new(
   #       assets_dir,
   #       LoadingState {
   #           texture_handle: None,
   #       },
   #       game_data,
   #   )?;
   # 
   #   //game.run();
   #   Ok(())
   # }
   ```

1. Wait for the asset to be loaded.

   When [`loader.load(..)`][doc_load] is used to load an [`Asset`][doc_asset], the method returns immediately with a handle for the asset. The asset loading is handled asynchronously in the background, so if the handle is used to retrieve the asset, such as with [`resources.get::<AssetStorage<Texture>>()`][doc_read_resource][`.get(texture_handle)`][doc_asset_get], it will return `None` until the `Texture` has finished loading.

   ```rust
   # use amethyst::prelude::*;
   # use amethyst::utils::application_root_dir;
   # use amethyst::{
   #   assets::{AssetStorage, DefaultLoader, Handle, Loader, ProgressCounter},
   #   renderer::{formats::texture::ImageFormat, Texture},
   # };
   # 
   # pub struct GameState {
   #   /// Handle to the player texture.
   #   texture_handle: Handle<Texture>,
   # }
   # 
   # impl SimpleState for GameState {}
   # 
   # pub struct LoadingState {
   #   /// Handle to the player texture.
   #   texture_handle: Option<Handle<Texture>>,
   # }
   # 
   impl SimpleState for LoadingState {
       //..
   #   fn on_start(&mut self, data: StateData<'_, GameData>) {
   #       let loader = data.resources.get::<DefaultLoader>().unwrap();
   #       let texture_handle = loader.load("player.png");
   # 
   #       self.texture_handle = Some(texture_handle);
   #   }
   # 
       fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
           let texture_storage = data.resources.get::<AssetStorage<Texture>>().unwrap();
           if let Some(texture) = texture_storage.get(self.texture_handle.as_ref().unwrap()) {
               println!("Loaded texture: {:?}", texture);
               Trans::Switch(Box::new(GameState {
                   texture_handle: self.texture_handle.take().expect(
                       "Expected `texture_handle` to exist when `progress_counter` is complete.",
                   ),
               }))
           } else {
               Trans::None
           }
       }
   }
   # fn main() -> amethyst::Result<()> {
   #   let app_root = application_root_dir()?;
   #   let assets_dir = app_root.join("assets");
   # 
   #   let game_data = DispatcherBuilder::default();
   #   let mut game = Application::new(
   #       assets_dir,
   #       LoadingState {
   #           texture_handle: None,
   #       },
   #       game_data,
   #   )?;
   # 
   #   //game.run();
   #   Ok(())
   # }
   ```

   The asset handle can now be used:

   ```rust
   # use amethyst::prelude::*;
   # use amethyst::utils::application_root_dir;
   # use amethyst::{
   #   assets::{AssetStorage, DefaultLoader, Handle, Loader, ProgressCounter},
   #   renderer::{formats::texture::ImageFormat, Texture},
   # };
   # 
   # pub struct LoadingState {
   #   /// Handle to the player texture.
   #   texture_handle: Option<Handle<Texture>>,
   # }
   # 
   # impl SimpleState for LoadingState {
   #   fn on_start(&mut self, data: StateData<'_, GameData>) {
   #       let loader = data.resources.get::<DefaultLoader>().unwrap();
   #       let texture_handle = loader.load("player.png");
   # 
   #       self.texture_handle = Some(texture_handle);
   #   }
   # 
   #   fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
   #       let texture_storage = data.resources.get::<AssetStorage<Texture>>().unwrap();
   #       if let Some(texture) = texture_storage.get(self.texture_handle.as_ref().unwrap()) {
   #           println!("Loaded texture: {:?}", texture);
   #           Trans::Switch(Box::new(GameState {
   #               texture_handle: self.texture_handle.take().expect(
   #                   "Expected `texture_handle` to exist when `progress_counter` is complete.",
   #               ),
   #           }))
   #       } else {
   #           Trans::None
   #       }
   #   }
   # }
   # 
   # pub struct GameState {
   #   /// Handle to the player texture.
   #   texture_handle: Handle<Texture>,
   # }
   impl SimpleState for GameState {
       fn on_start(&mut self, data: StateData<'_, GameData>) {
           // Create the player entity.
           data.world.push((self.texture_handle.clone(),));
       }
   }
   # fn main() -> amethyst::Result<()> {
   #   let app_root = application_root_dir()?;
   #   let assets_dir = app_root.join("assets");
   # 
   #   let game_data = DispatcherBuilder::default();
   #   let mut game = Application::new(
   #       assets_dir,
   #       LoadingState {
   #           texture_handle: None,
   #       },
   #       game_data,
   #   )?;
   # 
   #   //game.run();
   #   Ok(())
   # }
   # 
   ```

[doc_asset]: https://docs.amethyst.rs/master/amethyst_assets/trait.Asset.html
[doc_asset_get]: https://docs.amethyst.rs/master/amethyst_assets/struct.AssetStorage.html#method.get
[doc_load]: https://docs.amethyst.rs/master/amethyst_assets/struct.Loader.html#method.load
[doc_loader]: https://docs.amethyst.rs/master/amethyst_assets/struct.Loader.html
[doc_processor_system]: https://docs.amethyst.rs/master/amethyst_assets/struct.AssetProcessorSystem.html
[doc_read_resource]: https://docs.rs/specs/~0.16/specs/world/struct.World.html#method.read_resource
[doc_search_format]: https://docs.amethyst.rs/master/amethyst/?search=Format
