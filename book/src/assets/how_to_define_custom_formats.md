# How to Define Custom Formats

This guide explains how to define a new asset format. This will allow Amethyst to load assets stored in a particular encoding.

There is a trait in Amethyst for implementing a format: `Format<A: Asset::Data>`.
A blanket implementation will implement `Format::import` and we only need to implement `Format::import_simple`.

`Format` takes a type parameter for the asset data type it supports.

If you are defining a new format that may be useful to others, [please send us a PR!][gh_contributing]

1. Define a struct that represents the format.

   In most cases a unit struct is sufficient. This must implement `Default`, `Clone` and `Copy` for ergonomic usage.  It also must
   derive `Serialize`, `Deserialize` and `TypeUuid` for use in prefabs.

   ```rust
   use serde::{Deserialize, Serialize};
   use type_uuid::TypeUuid;

   /// Format for loading from `.mylang` files.
   #[derive(Default, Clone, Copy, Serialize, Deserialize, TypeUuid)]
   #[uuid = "00000000-0000-0000-0000-000000000002"] // replace with your own uuid
   pub struct MyLangFormat;
   ```

1. Implement the `Format` trait.

   This is where the logic to deserialize the [asset data type][bk_custom_assets] is provided.
   Fields of the format struct can be used to specify additional parameters for
   deserialization; use a unit struct if this is not needed.

   In this example the RON deserializer is used, though it is [already a supported format][doc_ron_format].

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
   # 
   # /// Format for loading from `.mylang` files.
   # #[derive(Default, Clone, Copy, Serialize, Deserialize, TypeUuid)]
   # #[uuid = "00000000-0000-0000-0000-000000000002"] // replace with your own uuid
   # pub struct MyLangFormat;

   use amethyst::assets::Format;
   use ron::de::Deserializer; // replace this with your formats deserializer

   // EnergyBlast could be EnergyBlastData here.
   impl Format<EnergyBlast> for MyLangFormat {
       fn name(&self) -> &'static str {
           "MyLangFormat"
       }

       fn import_simple(&self, bytes: Vec<u8>) -> amethyst::Result<EnergyBlast> {
           let mut deserializer = Deserializer::from_bytes(&bytes)?;
           let val = EnergyBlast::deserialize(&mut deserializer)?;
           deserializer.end()?;

           Ok(val)
       }
   }

   use amethyst::assets as amethyst_assets;
   amethyst::assets::register_importer!(".mylang", MyLangFormat);
   ```

   The custom format can now be used:

   ```rust
   # use amethyst::assets::{Asset, AssetStorage, Format, Handle, ProcessingState, ProgressCounter};
   # use ron::de::Deserializer;
   # use serde::{Deserialize, Serialize};
   # use type_uuid::TypeUuid;
   # /// Custom asset representing an energy blast.
   # #[derive(Clone, Debug, Default, Serialize, Deserialize, TypeUuid)]
   # #[uuid = "00000000-0000-0000-0000-000000000001"]
   # pub struct EnergyBlast {
   #   /// How much HP to subtract.
   #   pub hp_damage: u32,
   #   /// How much MP to subtract.
   #   pub mp_damage: u32,
   # }
   # 
   # impl Asset for EnergyBlast {
   #   type Data = Self;
   # 
   #   fn name() -> &'static str {
   #       "EnergyBlast"
   #   }
   # }
   # 
   # pub struct LoadingState {
   #   /// Handle to the energy blast.
   #   energy_blast_handle: Option<Handle<EnergyBlast>>,
   # }
   # 
   # /// Format for loading from `.mylang` files.
   # #[derive(Default, Debug, Clone, Serialize, Deserialize, TypeUuid)]
   # #[uuid = "00000000-0000-0000-0000-000000000002"] // replace with your own uuid
   # pub struct MyLangFormat;
   # 
   # impl Format<EnergyBlast> for MyLangFormat {
   #   fn name(&self) -> &'static str {
   #       "MyLangFormat"
   #   }
   # 
   #   fn import_simple(&self, bytes: Vec<u8>) -> amethyst::Result<EnergyBlast> {
   #       let mut deserializer = Deserializer::from_bytes(&bytes)?;
   #       let val = EnergyBlast::deserialize(&mut deserializer)?;
   #       deserializer.end()?;
   # 
   #       Ok(val)
   #   }
   # }
   # 

   use amethyst::assets::{DefaultLoader, Loader, LoaderBundle};
   use amethyst::{
       ecs::DispatcherBuilder, utils::application_root_dir, Application, GameData, SimpleState,
       SimpleTrans, StateData, Trans,
   };

   impl SimpleState for LoadingState {
       fn on_start(&mut self, data: StateData<'_, GameData>) {
           let loader = data.resources.get::<DefaultLoader>().unwrap();
           let energy_blast_handle = loader.load("energy_blast.mylang");

           self.energy_blast_handle = Some(energy_blast_handle);
       }
   }

   fn main() -> amethyst::Result<()> {
       amethyst::start_logger(Default::default());
       let app_root = application_root_dir()?;
       let assets_dir = app_root.join("assets");

       let mut game_data = DispatcherBuilder::default();
       game_data.add_bundle(LoaderBundle);

       let mut game = Application::new(
           assets_dir,
           LoadingState {
               energy_blast_handle: None,
           },
           game_data,
       )?;

       //game.run();
       Ok(())
   }
   ```

[bk_custom_assets]: how_to_define_custom_assets.html
[doc_ron_format]: https://docs.amethyst.rs/stable/amethyst_assets/struct.RonFormat.html
[gh_contributing]: https://github.com/amethyst/amethyst/blob/master/docs/CONTRIBUTING.md
