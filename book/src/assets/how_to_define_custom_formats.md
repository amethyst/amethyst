# How to Define Custom Formats

This guide explains how to define a new asset format. This will allow Amethyst to load assets stored in a particular encoding.

There is a trait in Amethyst for implementing a format: `Format<A: Asset::Data>`.
`Format` provides a loading implementation that provides detection when an asset should be reloaded for [hot reloading][doc_hrs]; you don't need to implement it since it has a default implementation.
A blanket implementation will implement `Format::import` and we only need to implement 
`Format::import_simple`.

`Format` takes a type parameter for the asset data type it supports. This guide covers a type 
parameterized implementation of `Format<D>` where `D` is an arbitrary `Asset::Data`, so we can
reuse it for any asset which can be loaded from deserializable asset data.

If you are defining a new format that may be useful to others, [please send us a PR!][gh_contributing]

1. Define a struct that represents the format.

    In most cases a unit struct is sufficient. When possible, this should implement `Clone` and `Copy` for ergonomic usage.

    ```rust,edition2018,no_run,noplaypen
    /// Format for loading from `.mylang` files.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct MyLangFormat;
    ```

2. Implement the `Format` trait.

    This is where the logic to deserialize the [asset data type][bk_custom_assets] is provided. 
    Fields of the format struct can be used to specify additional parameters for 
    deserialization; use a unit struct if this is not needed.

    In this example the RON deserializer is used, though it is [already a supported format][doc_ron_format].

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    # extern crate ron;
    # extern crate serde;
    #
    use amethyst::{
        error::Error,
        assets::{Asset, Format},
    };
    use serde::Deserialize;
    use ron::de::Deserializer; // Replace this in your implementation.

    /// Format for loading from `.mylang` files.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct MyLangFormat;

    impl<D> Format<D> for MyLangFormat
    where
        D: for<'a> Deserialize<'a> + Send + Sync + 'static,
    {
        fn name(&self) -> &'static str {
            "MyLangFormat"
        }

        fn import_simple(&self, bytes: Vec<u8>) -> Result<D, Error> {
            let mut deserializer = Deserializer::from_bytes(&bytes)?;
            let val = D::deserialize(&mut deserializer)?;
            deserializer.end()?;

            Ok(val)
        }
    }
    ```

    The custom format can now be used:

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    # extern crate ron;
    # extern crate serde;
    # extern crate serde_derive;
    #
    # use amethyst::{
    #     error::Error,
    #     assets::{
    #         Asset, AssetStorage, Handle, Loader, Processor, ProgressCounter,
    #         ProcessingState, Format,
    #     },
    #     ecs::{VecStorage, World, WorldExt},
    #     prelude::*,
    #     utils::application_root_dir,
    # };
    # use ron::de::Deserializer;
    # use serde::Deserialize as DeserializeTrait;
    # use serde_derive::{Deserialize, Serialize};
    #
    # /// Custom asset representing an energy blast.
    # #[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
    # pub struct EnergyBlast {
    #     /// How much HP to subtract.
    #     pub hp_damage: u32,
    #     /// How much MP to subtract.
    #     pub mp_damage: u32,
    # }
    #
    # /// A handle to a `EnergyBlast` asset.
    # pub type EnergyBlastHandle = Handle<EnergyBlast>;
    #
    # impl Asset for EnergyBlast {
    #     const NAME: &'static str = "my_crate::EnergyBlast";
    #     type Data = Self;
    #     type HandleStorage = VecStorage<EnergyBlastHandle>;
    # }
    #
    # impl From<EnergyBlast> for Result<ProcessingState<EnergyBlast>, Error> {
    #     fn from(energy_blast: EnergyBlast) -> Result<ProcessingState<EnergyBlast>, Error> {
    #       Ok(ProcessingState::Loaded(energy_blast))
    #     }
    # }
    #
    # pub struct LoadingState {
    #     /// Tracks loaded assets.
    #     progress_counter: ProgressCounter,
    #     /// Handle to the energy blast.
    #     energy_blast_handle: Option<EnergyBlastHandle>,
    # }
    #
    # /// Format for loading from `.mylang` files.
    #  #[derive(Clone, Copy, Debug, Default)]
    #  pub struct MyLangFormat;
    #
    #  impl<D> Format<D> for MyLangFormat
    #  where
    #      D: for<'a> DeserializeTrait<'a> + Send + Sync + 'static,
    #  {
    #      fn name(&self) -> &'static str {
    #          "MyLangFormat"
    #      }
    #
    #      fn import_simple(&self, bytes: Vec<u8>) -> Result<D, Error> {
    #          let mut deserializer = Deserializer::from_bytes(&bytes)?;
    #          let val = D::deserialize(&mut deserializer)?;
    #          deserializer.end()?;
    #
    #          Ok(val)
    #      }
    #  }
    #
    impl SimpleState for LoadingState {
        fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
            let loader = &data.world.read_resource::<Loader>();
            let energy_blast_handle = loader.load(
                "energy_blast.mylang",
                MyLangFormat,
                &mut self.progress_counter,
                &data.world.read_resource::<AssetStorage<EnergyBlast>>(),
            );

            self.energy_blast_handle = Some(energy_blast_handle);
        }
    #
    #     fn update(
    #         &mut self,
    #         _data: &mut StateData<'_, GameData<'_, '_>>,
    #     ) -> SimpleTrans {
    #         Trans::Quit
    #     }
    }
    #
    # fn main() -> amethyst::Result<()> {
    #     amethyst::start_logger(Default::default());
    #     let app_root = application_root_dir()?;
    #     let assets_dir = app_root.join("assets");
    #
    #     let game_data = GameDataBuilder::default()
    #         .with(Processor::<EnergyBlast>::new(), "", &[]);
    #     let mut game = Application::new(
    #         assets_dir,
    #         LoadingState {
    #             progress_counter: ProgressCounter::new(),
    #             energy_blast_handle: None,
    #         },
    #         game_data,
    #     )?;
    #
    #     game.run();
    #     Ok(())
    # }
    ```

[bk_custom_assets]: how_to_define_custom_assets.html
[doc_hrs]: https://docs.amethyst.rs/stable/amethyst_assets/struct.HotReloadStrategy.html
[doc_ron_format]: https://docs.amethyst.rs/stable/amethyst_assets/struct.RonFormat.html
[gh_contributing]: https://github.com/amethyst/amethyst/blob/master/docs/CONTRIBUTING.md
