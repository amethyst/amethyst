//! Example showing how to load a Locale file as an Asset using the Loader.

extern crate amethyst;

use amethyst::Error;
use amethyst::assets::{AssetStorage, Handle, Loader, Processor, ProgressCounter};
use amethyst::locale::*;
use amethyst::prelude::*;

struct Example {
    progress_counter: Option<ProgressCounter>,
    handle: Option<Handle<Locale>>,
}

impl Example {
    pub fn new() -> Self {
        Example { progress_counter: None, handle: None }
    }
}

impl<'a,'b> State<GameData<'a, 'b>> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        data.world.add_resource(AssetStorage::<Locale>::new());
        let loader = data.world.read_resource::<Loader>();
        let mut progress_counter = ProgressCounter::default();
        self.handle = Some(loader.load(
            "locale/locale.ftl",
            LocaleFormat,
            (),
            &mut progress_counter,
            &data.world.read_resource(),
        ));
        self.progress_counter = Some(progress_counter);
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        // Check if the locale has been loaded.
        if self.progress_counter.as_ref().unwrap().is_complete() {
            let store = data.world.read_resource::<AssetStorage<Locale>>();
            if let Some(locale) = self.handle.as_ref().and_then(|h| store.get(h)) {
                println!(
                    "{}",
                    locale
                        .context
                        .get_message("hello")
                        .and_then(|msg| locale.context.format(msg, None))
                        .unwrap()
                );
                println!(
                    "{}",
                    locale
                        .context
                        .get_message("bye")
                        .and_then(|msg| locale.context.format(msg, None))
                        .unwrap()
                );
                Trans::Quit
            } else {
                Trans::None
            }
        }else{
            Trans::None
        }
    }
}

fn main() -> Result<(), Error> {
    let resources_directory = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

    let game_data = GameDataBuilder::default()
        .with(Processor::<Locale>::new(), "proc", &[]);

    let mut game = Application::new(resources_directory, Example::new(),game_data)?;
    game.run();
    Ok(())
}