# How to Define Custom Assets

This guide explains how to define a new asset type to be used in an Amethyst application. If you are defining a new asset type that may be useful to others, [please send us a PR!][gh_contributing]

1. Define the type and handle for your asset.

    ```rust,edition2018,ignore,noplaypen
    # extern crate amethyst;
    # extern crate serde_derive;
    #
    use amethyst::{
        assets::Handle,
        ecs::VecStorage,
    };

    /// Custom asset representing an energy blast.
    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct EnergyBlast {
        /// How much HP to subtract.
        pub hp_damage: u32,
        /// How much MP to subtract.
        pub mp_damage: u32,
    }

    /// A handle to a `EnergyBlast` asset.
    pub type EnergyBlastHandle = Handle<EnergyBlast>;
    ```

2. Define the type that represents the serializable form of the asset.

    The serializable type can be one of:

    * The asset type itself, in which case you simply derive `Serialize` and `Deserialize` on the type:

        ```rust,ignore
        #[derive(Serialize, Deserialize, ..)]
        pub struct EnergyBlast { .. }
        ```

    * An enum with different variants &ndash; each for a different data layout:

        ```rust,edition2018,ignore,noplaypen
        # extern crate serde_derive;
        #
        # use serde_derive::{Deserialize, Serialize};

        /// Separate serializable type to support different versions
        /// of energy blast configuration.
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub enum EnergyBlastData {
            /// Early version only could damage HP.
            Version1 { hp_damage: u32 },
            /// Add support for subtracting MP.
            Version2 { hp_damage: u32, mp_damage: u32 },
        }
        ```

3. Implement the [`Asset`][doc_asset] trait on the asset type.

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    # extern crate serde_derive;
    #
    # use amethyst::{
    #     assets::{Asset, Handle},
    #     ecs::VecStorage,
    # };
    # use serde_derive::{Deserialize, Serialize};
    #
    # /// Custom asset representing an energy blast.
    # #[derive(Clone, Debug, Default, PartialEq, Eq)]
    # pub struct EnergyBlast {
    #     /// How much HP to subtract.
    #     pub hp_damage: u32,
    #     /// How much MP to subtract.
    #     pub mp_damage: u32,
    # }
    #
    impl Asset for EnergyBlast {
        const NAME: &'static str = "my_crate::EnergyBlast";
        // use `Self` if the type is directly serialized.
        type Data = EnergyBlastData;
        type HandleStorage = VecStorage<EnergyBlastHandle>;
    }
    #
    # /// A handle to a `EnergyBlast` asset.
    # pub type EnergyBlastHandle = Handle<EnergyBlast>;
    #
    # /// Separate serializable type to support different versions
    # /// of energy blast configuration.
    # #[derive(Clone, Debug, Deserialize, Serialize)]
    # pub enum EnergyBlastData {
    #     /// Early version only could damage HP.
    #     Version1 { hp_damage: u32 },
    #     /// Add support for subtracting MP.
    #     Version2 { hp_damage: u32, mp_damage: u32 },
    # }
    ```

4. Implement the [`ProcessableAsset`][doc_processable_asset] trait, providing the conversion function for `A::Data` into a `ProcessingState<A>` result.

    The [`Processor<A>` system][doc_processor_system] uses this trait to convert the deserialized asset data into the asset.

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    # extern crate serde_derive;
    #
    # use amethyst::{
    #     error::Error,
    #     assets::{Asset, Handle, ProcessingState, ProcessableAsset},
    #     ecs::VecStorage,
    # };
    # use serde_derive::{Deserialize, Serialize};
    #
    # /// Custom asset representing an energy blast.
    # #[derive(Clone, Debug, Default, PartialEq, Eq)]
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
    #     // use `Self` if the type is directly serialized.
    #     type Data = EnergyBlastData;
    #     type HandleStorage = VecStorage<EnergyBlastHandle>;
    # }
    #
    # /// Separate serializable type to support different versions
    # /// of energy blast configuration.
    # #[derive(Clone, Debug, Deserialize, Serialize)]
    # pub enum EnergyBlastData {
    #     /// Early version only could damage HP.
    #     Version1 { hp_damage: u32 },
    #     /// Add support for subtracting MP.
    #     Version2 { hp_damage: u32, mp_damage: u32 },
    # }
    #
    impl ProcessableAsset for EnergyBlast {
        fn process(energy_blast_data: Self::Data) -> Result<ProcessingState<Self>, Error> {
            match energy_blast_data {
                EnergyBlastData::Version1 { hp_damage } => {
                    Ok(ProcessingState::Loaded(Self {
                        hp_damage,
                        ..Default::default()
                    }))
                }
                EnergyBlastData::Version2 { hp_damage, mp_damage } => {
                    Ok(ProcessingState::Loaded(Self {
                        hp_damage,
                        mp_damage,
                    }))
                }
            }
        }
    }
    ```

    If your asset is stored using one of the existing supported formats such as RON or JSON, it can now be used:

    ```rust,edition2018,no_run,noplaypen
    # extern crate amethyst;
    # extern crate serde_derive;
    #
    # use amethyst::{
    #     error::Error,
    #     assets::{AssetStorage, Loader, ProcessableAsset, ProcessingState, ProgressCounter, RonFormat},
    #     ecs::{World, WorldExt},
    #     prelude::*,
    #     utils::application_root_dir,
    # };
    # use serde_derive::{Deserialize, Serialize};
    #
    # use amethyst::{
    #     assets::{Asset, Handle},
    #     ecs::VecStorage,
    # };
    #
    # /// Custom asset representing an energy blast.
    # #[derive(Clone, Debug, Default, PartialEq, Eq)]
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
    # /// Separate serializable type to support different versions
    # /// of energy blast configuration.
    # #[derive(Clone, Debug, Deserialize, Serialize)]
    # pub enum EnergyBlastData {
    #     /// Early version only could damage HP.
    #     Version1 { hp_damage: u32 },
    #     /// Add support for subtracting MP.
    #     Version2 { hp_damage: u32, mp_damage: u32 },
    # }
    #
    # impl Asset for EnergyBlast {
    #     const NAME: &'static str = "my_crate::EnergyBlast";
    #     // use `Self` if the type is directly serialized.
    #     type Data = EnergyBlastData;
    #     type HandleStorage = VecStorage<EnergyBlastHandle>;
    # }
    #
    # impl ProcessableAsset for EnergyBlast {
    #     fn process(energy_blast_data: Self::Data) -> Result<ProcessingState<Self>, Error> {
    #         match energy_blast_data {
    #             EnergyBlastData::Version1 { hp_damage } => {
    #                 Ok(ProcessingState::Loaded(Self {
    #                     hp_damage,
    #                     ..Default::default()
    #                 }))
    #             }
    #             EnergyBlastData::Version2 { hp_damage, mp_damage } => {
    #                 Ok(ProcessingState::Loaded(Self {
    #                     hp_damage,
    #                     mp_damage,
    #                 }))
    #             }
    #         }
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
    impl SimpleState for LoadingState {
        fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
            let loader = &data.world.read_resource::<Loader>();
            let energy_blast_handle = loader.load(
                "energy_blast.ron",
                RonFormat,
                &mut self.progress_counter,
                &data.world.read_resource::<AssetStorage<EnergyBlast>>(),
            );

            self.energy_blast_handle = Some(energy_blast_handle);
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
    #           energy_blast_handle: None,
    #       },
    #       game_data,
    #   )?;
    #
    #   game.run();
    #   Ok(())
    # }
    ```

    If the asset data is stored in a format that is not supported by Amethyst, a [custom format][bk_custom_formats] can be implemented and provided to the `Loader` to load the asset data.

[bk_custom_formats]: how_to_define_custom_formats.html
[doc_asset]: https://docs.amethyst.rs/master/amethyst_assets/trait.Asset.html
[doc_processor_system]: https://docs.amethyst.rs/master/amethyst_assets/struct.Processor.html
[doc_processable_asset]: https://docs.amethyst.rs/master/amethyst_assets/trait.ProcessableAsset.html
[gh_contributing]: https://github.com/amethyst/amethyst/blob/master/docs/CONTRIBUTING.md
