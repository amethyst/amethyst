use amethyst::{
    assets::{
        self, Asset, AssetStorage, Handle, Loader, ProcessingState, Processor, ProgressCounter,
        ResultExt, SimpleFormat,
    },
    ecs::VecStorage,
    prelude::*,
    utils::application_root_dir,
};
use log::info;
use ron::de::Deserializer;
use serde::{Deserialize, Serialize};

/// Custom asset representing an energy blast.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EnergyBlast {
    /// How much HP to subtract.
    pub hp_damage: u32,
    /// How much MP to subtract.
    pub mp_damage: u32,
}

/// A handle to a `EnergyBlast` asset.
pub type EnergyBlastHandle = Handle<EnergyBlast>;

impl Asset for EnergyBlast {
    const NAME: &'static str = "my_crate::EnergyBlast";
    type Data = Self;
    type HandleStorage = VecStorage<EnergyBlastHandle>;
}

impl From<EnergyBlast> for assets::Result<ProcessingState<EnergyBlast>> {
    fn from(energy_blast: EnergyBlast) -> assets::Result<ProcessingState<EnergyBlast>> {
        Ok(ProcessingState::Loaded(energy_blast))
    }
}

pub struct LoadingState {
    /// Tracks loaded assets.
    progress_counter: ProgressCounter,
    /// Handle to the energy blast.
    energy_blast_handle: Option<EnergyBlastHandle>,
}

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
        let val =
            A::Data::deserialize(&mut deserializer).chain_err(|| "Failed parsing MyLang file")?;
        deserializer
            .end()
            .chain_err(|| "Failed parsing MyLang file")?;

        Ok(val)
    }
}

#[derive(Debug)]
struct CodeSource;

impl assets::Source for CodeSource {
    fn modified(&self, _path: &str) -> assets::Result<u64> {
        Ok(0)
    }
    fn load(&self, _path: &str) -> assets::Result<Vec<u8>> {
        let bytes = "EnergyBlast(hp_damage: 10, mp_damage: 10)"
            .as_bytes()
            .to_vec();
        Ok(bytes)
    }
}

impl SimpleState for LoadingState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        {
            let mut loader = data.world.write_resource::<Loader>();
            loader.add_source("code_source", CodeSource);
        }

        let loader = &data.world.read_resource::<Loader>();

        let energy_blast_handle = loader.load_from(
            "energy_blast.mylang",
            self::MyLangFormat,
            (),
            "code_source",
            &mut self.progress_counter,
            &data.world.read_resource::<AssetStorage<EnergyBlast>>(),
        );

        self.energy_blast_handle = Some(energy_blast_handle);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if self.progress_counter.is_complete() {
            let energy_blast_assets = data.world.read_resource::<AssetStorage<EnergyBlast>>();
            let energy_blast = energy_blast_assets
                .get(
                    self.energy_blast_handle
                        .as_ref()
                        .expect("Expected energy_blast_handle to be set."),
                )
                .expect("Expected energy blast to be loaded.");
            info!("Loaded energy blast: {:?}", energy_blast);
            Trans::Quit
        } else {
            Trans::None
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());
    let app_root = application_root_dir()?;
    let assets_dir = app_root.join("assets");

    let game_data = GameDataBuilder::default().with(Processor::<EnergyBlast>::new(), "", &[]);
    let mut game = Application::new(
        assets_dir,
        LoadingState {
            progress_counter: ProgressCounter::new(),
            energy_blast_handle: None,
        },
        game_data,
    )?;

    game.run();
    Ok(())
}
