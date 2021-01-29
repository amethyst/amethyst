# How to Define Custom Assets

This guide explains how to define a new asset type to use in an Amethyst application. If you are defining a new asset type that may be useful to others, [please send us a PR!][gh_contributing]

1. Define the type and handle for your asset.

   ```rust
   /// Custom asset representing an energy blast.
   #[derive(Clone, Debug, Default)]
   pub struct EnergyBlast {
       pub hp_damage: u32,
       pub mp_damage: u32,
   }
   ```

1. Define the type that represents the serialized form of the asset.

   The serialized type can be one of:

   - The asset type itself, in which case you derive `Serialize`, `Deserialize` and `TypeUuid` on the type:

     ```rust
     use serde::{Deserialize, Serialize};
     use type_uuid::TypeUuid;

     #[derive(Clone, Debug, Default, Serialize, Deserialize, TypeUuid)]
     #[uuid = "00000000-0000-0000-0000-000000000001"] // generate this uuid yourself
     pub struct EnergyBlast {
         pub hp_damage: u32,
         pub mp_damage: u32,
     }
     ```

   - An enum with different variants – each for a different data layout:

     ```rust
     # use serde::{Deserialize, Serialize};

     /// Separate serializable type to support different versions
     /// of energy blast configuration.
     #[derive(Clone, Debug, Serialize, Deserialize)]
     pub enum EnergyBlastData {
         /// Early version only could damage HP.
         Version1 { hp_damage: u32 },
         /// Add support for subtracting MP.
         Version2 { hp_damage: u32, mp_damage: u32 },
     }
     ```

1. Implement the [`Asset`][doc_asset] trait on the asset type.

   ```rust
   # use amethyst::assets::{Asset, Handle};
   # use serde::{Deserialize, Serialize};
   # use type_uuid::TypeUuid;
   # /// Custom asset representing an energy blast.
   # #[derive(Clone, Debug, Default, Serialize, Deserialize, TypeUuid)]
   # #[uuid = "00000000-0000-0000-0000-000000000001"]
   # pub struct EnergyBlast {
   #   pub hp_damage: u32,
   #   pub mp_damage: u32,
   # }
   # 
   # /// Separate serializable type to support different versions
   # /// of energy blast configuration.
   # #[derive(Clone, Debug, Deserialize, Serialize)]
   # pub enum EnergyBlastData {
   #   /// Early version only could damage HP.
   #   Version1 { hp_damage: u32 },
   #   /// Add support for subtracting MP.
   #   Version2 { hp_damage: u32, mp_damage: u32 },
   # }

   impl Asset for EnergyBlast {
       // use `Self` if the type is directly serialized.
       type Data = EnergyBlastData;

       fn name() -> &'static str {
           "EnergyBlast"
       }
   }
   ```

1. Implement the [`ProcessableAsset`][doc_processable_asset] trait, providing the conversion function for `A::Data` into a `ProcessingState<A>` result.

   The [`AssetProcessorSystem<A>` system][doc_processor_system] uses this trait to convert the deserialized asset data into the asset.

   ```rust
   # use amethyst::assets::{Asset, Handle};
   # use serde::{Deserialize, Serialize};
   # use type_uuid::TypeUuid;
   # 
   # /// Custom asset representing an energy blast.
   # #[derive(Clone, Debug, Default, Serialize, Deserialize, TypeUuid)]
   # #[uuid = "00000000-0000-0000-0000-000000000001"]
   # pub struct EnergyBlast {
   #   pub hp_damage: u32,
   #   pub mp_damage: u32,
   # }
   # 
   # /// Separate serializable type to support different versions
   # /// of energy blast configuration.
   # #[derive(Clone, Debug, Deserialize, Serialize)]
   # pub enum EnergyBlastData {
   #   /// Early version only could damage HP.
   #   Version1 { hp_damage: u32 },
   #   /// Add support for subtracting MP.
   #   Version2 { hp_damage: u32, mp_damage: u32 },
   # }
   # 
   # impl Asset for EnergyBlast {
   #   // use `Self` if the type is directly serialized.
   #   type Data = EnergyBlastData;
   # 
   #   fn name() -> &'static str {
   #       "EnergyBlast"
   #   }
   # }

   use amethyst::assets::{AssetStorage, LoadHandle, ProcessableAsset, ProcessingState};

   impl ProcessableAsset for EnergyBlast {
       fn process(
           energy_blast_data: Self::Data,
           _storage: &mut AssetStorage<Self>,
           _handle: &LoadHandle,
       ) -> amethyst::Result<ProcessingState<Self::Data, Self>> {
           match energy_blast_data {
               EnergyBlastData::Version1 { hp_damage } => Ok(ProcessingState::Loaded(Self {
                   hp_damage,
                   ..Default::default()
               })),
               EnergyBlastData::Version2 {
                   hp_damage,
                   mp_damage,
               } => Ok(ProcessingState::Loaded(Self {
                   hp_damage,
                   mp_damage,
               })),
           }
       }
   }
   ```

   If your asset is stored using one of the existing supported formats such as RON or JSON, it can now be used:

   ```rust
   # use amethyst::assets::{Asset, Handle};
   # use serde::{Deserialize, Serialize};
   # use type_uuid::TypeUuid;
   # 
   # /// Custom asset representing an energy blast.
   # #[derive(Clone, Debug, Default, Serialize, Deserialize, TypeUuid)]
   # #[uuid = "00000000-0000-0000-0000-000000000001"]
   # pub struct EnergyBlast {
   #   pub hp_damage: u32,
   #   pub mp_damage: u32,
   # }
   # 
   # /// Separate serializable type to support different versions
   # /// of energy blast configuration.
   # #[derive(Clone, Debug, Deserialize, Serialize)]
   # pub enum EnergyBlastData {
   #   /// Early version only could damage HP.
   #   Version1 { hp_damage: u32 },
   #   /// Add support for subtracting MP.
   #   Version2 { hp_damage: u32, mp_damage: u32 },
   # }
   # 
   # impl Asset for EnergyBlast {
   #   // use `Self` if the type is directly serialized.
   #   type Data = EnergyBlastData;
   # 
   #   fn name() -> &'static str {
   #       "EnergyBlast"
   #   }
   # }
   # 
   # use amethyst::assets::{AssetStorage, LoadHandle, ProcessableAsset, ProcessingState};
   # 
   # impl ProcessableAsset for EnergyBlast {
   #   fn process(
   #       energy_blast_data: Self::Data,
   #       _storage: &mut AssetStorage<Self>,
   #       _handle: &LoadHandle,
   #   ) -> amethyst::Result<ProcessingState<Self::Data, Self>> {
   #       match energy_blast_data {
   #           EnergyBlastData::Version1 { hp_damage } => Ok(ProcessingState::Loaded(Self {
   #               hp_damage,
   #               ..Default::default()
   #           })),
   #           EnergyBlastData::Version2 {
   #               hp_damage,
   #               mp_damage,
   #           } => Ok(ProcessingState::Loaded(Self {
   #               hp_damage,
   #               mp_damage,
   #           })),
   #       }
   #   }
   # }

   pub struct LoadingState {
       /// Handle to the energy blast.
       energy_blast_handle: Option<Handle<EnergyBlast>>,
   }

   use amethyst::assets::{DefaultLoader, Loader};
   use amethyst::{
       ecs::DispatcherBuilder, utils::application_root_dir, Application, GameData, SimpleState,
       SimpleTrans, StateData, Trans,
   };

   impl SimpleState for LoadingState {
       fn on_start(&mut self, data: StateData<'_, GameData>) {
           let loader = data.resources.get::<DefaultLoader>().unwrap();
           let energy_blast_handle = loader.load("energy_blast.ron");

           self.energy_blast_handle = Some(energy_blast_handle);
       }

       fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
           let energy_blast_assets = data.resources.get::<AssetStorage<EnergyBlast>>().unwrap();
           if let Some(energy_blast) =
               energy_blast_assets.get(self.energy_blast_handle.as_ref().unwrap())
           {
               println!("Loaded energy blast: {:?}", energy_blast);
               Trans::Quit
           } else {
               Trans::None
           }
       }
   }

   fn main() -> amethyst::Result<()> {
       let app_root = application_root_dir()?;
       let assets_dir = app_root.join("assets");

       let game_data = DispatcherBuilder::default();
       let mut game = Application::new(
           assets_dir,
           LoadingState {
               energy_blast_handle: None,
           },
           game_data,
       )?;

       // uncomment to run this example
       // game.run();
       Ok(())
   }
   ```

   If the asset data is stored in a format that is not supported by Amethyst, a [custom format][bk_custom_formats] can be implemented and provided to the `Loader` to load the asset data.

[bk_custom_formats]: how_to_define_custom_formats.html
[doc_asset]: https://docs.amethyst.rs/master/amethyst_assets/trait.Asset.html
[doc_processable_asset]: https://docs.amethyst.rs/master/amethyst_assets/trait.ProcessableAsset.html
[doc_processor_system]: https://docs.amethyst.rs/master/amethyst_assets/struct.AssetProcessorSystem.html
[gh_contributing]: https://github.com/amethyst/amethyst/blob/master/docs/CONTRIBUTING.md
