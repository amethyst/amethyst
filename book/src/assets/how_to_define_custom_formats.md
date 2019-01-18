# How to Define Custom Formats

This guide explains how to define a new asset format. This will allow Amethyst to load assets stored in a particular encoding.

There are two formatting traits in Amethyst &ndash; `Format<A: Asset>` and `SimpleFormat<A: Asset>`. `Format` requests a loading implementation that provides detection when an asset should be reloaded for [hot reloading][doc_hrs]; `SimpleFormat` requires a loading implementation. A blanket implementation will implement `Format` for all `SimpleFormat` implementations, and provides hot reloading detection based on file modification time &ndash; this is sufficient for most applications that require hot reloading. This guide covers implementation of the `SimpleFormat` trait.

`SimpleFormat` takes a type parameter for the asset type it supports. This guide covers a type parameterized implementation of `SimpleFormat<A: Asset>` for all `A`, as it is easier to deduce the specific implementation from a parameterized implementation than the other way around.

If you are defining a new format that may be useful to others, [please send us a PR!][gh_contributing]

1. Define a struct that represents the format.

    In most cases a unit struct is sufficient. When possible, this should implement `Clone` and `Copy` for ergonomic usage.

    ```rust,edition2018,no_run,noplaypen
    /// Format for loading from `.mylang` files.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct MyLangFormat;
    ```

2. Implement the `SimpleFormat` trait.

    This is where the logic to deserialize the [asset data type][bk_custom_assets] is provided. The `Options` associated type can be used to specify additional parameters for deserialization; use the empty tuple `()` if this is not needed.

    In this example the RON deserializer is used, though it is [already a supported format][doc_ron_format].

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    # extern crate ron;
    # extern crate serde;
    #
    use amethyst::{
        assets::{self, Asset, ResultExt, SimpleFormat},
    };
    use serde::Deserialize;
    use ron::de::Deserializer; // Replace this in your implementation.

    /// Format for loading from `.mylang` files.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct MyLangFormat;

    impl<A> SimpleFormat<A> for MyLangFormat
    where
        A: Asset,
        A::Data: for<'a> Deserialize<'a> + Send + Sync + 'static,
    {
        const NAME: &'static str = "MyLang";

        // If the deserializer implementation takes parameters,
        // the parameter type may be specified here.
        type Options = ();

        fn import(&self, bytes: Vec<u8>, _: ()) -> Result<A::Data, assets::Error> {
            let mut deserializer =
                Deserializer::from_bytes(&bytes).chain_err(|| "Failed deserializing MyLang file")?;
            let val = A::Data::deserialize(&mut deserializer).chain_err(|| "Failed parsing MyLang file")?;
            deserializer.end().chain_err(|| "Failed parsing MyLang file")?;

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
    #     assets::{
    #         self, Asset, AssetStorage, Handle, Loader, Processor, ProgressCounter,
    #         ProcessingState, ResultExt, SimpleFormat,
    #     },
    #     ecs::VecStorage,
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
    # impl From<EnergyBlast> for assets::Result<ProcessingState<EnergyBlast>> {
    #     fn from(energy_blast: EnergyBlast) -> assets::Result<ProcessingState<EnergyBlast>> {
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
    # #[derive(Clone, Copy, Debug, Default)]
    # pub struct MyLangFormat;
    #
    # impl<A> SimpleFormat<A> for MyLangFormat
    # where
    #     A: Asset,
    #     A::Data: for<'a> DeserializeTrait<'a> + Send + Sync + 'static,
    # {
    #     const NAME: &'static str = "MyLang";
    #
    #     // If the deserializer implementation takes parameters,
    #     // the parameter type may be specified here.
    #     type Options = ();
    #
    #     fn import(&self, bytes: Vec<u8>, _: ()) -> Result<A::Data, assets::Error> {
    #         let mut deserializer = Deserializer::from_bytes(&bytes)
    #             .chain_err(|| "Failed deserializing MyLang file")?;
    #         let val = A::Data::deserialize(&mut deserializer)
    #             .chain_err(|| "Failed parsing MyLang file")?;
    #         deserializer.end().chain_err(|| "Failed parsing MyLang file")?;
    #
    #         Ok(val)
    #     }
    # }
    #
    impl SimpleState for LoadingState {
        fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
            let loader = &data.world.read_resource::<Loader>();
            let energy_blast_handle = loader.load(
                "energy_blast.mylang",
                MyLangFormat,
                (),
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
[doc_hrs]: https://www.amethyst.rs/doc/latest/doc/amethyst_assets/struct.HotReloadStrategy.html
[doc_ron_format]: https://www.amethyst.rs/doc/latest/doc/amethyst_assets/struct.RonFormat.html
[gh_contributing]: https://github.com/amethyst/amethyst/blob/master/docs/CONTRIBUTING.md
