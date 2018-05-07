//! Example showing how to load a Locale file as an Asset using the Loader.

extern crate amethyst;

use amethyst::Error;
use amethyst::assets::{AssetStorage, Handle, Loader, Processor};
use amethyst::locale::*;
use amethyst::prelude::*;

struct Example {
    handle: Option<Handle<Locale>>,
}

impl Example {
    pub fn new() -> Self {
        Example { handle: None }
    }
}

impl State for Example {
    fn on_start(&mut self, world: &mut World) {
        world.add_resource(AssetStorage::<Locale>::new());
        let loader = world.read_resource::<Loader>();
        self.handle = Some(loader.load(
            "locale/locale.ftl",
            LocaleFormat,
            (),
            (),
            &world.read_resource(),
        ));
    }

    fn update(&mut self, world: &mut World) -> Trans {
        let store = world.read_resource::<AssetStorage<Locale>>();

        // Check if the locale has been loaded.
        // If you are doing this for multiple assets, you should be using `ProgressCounter`.
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
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Initialisation error: {}", error);
        ::std::process::exit(1);
    }
}
fn run() -> Result<(), Error> {
    let resources_directory = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));
    let mut game = Application::build(resources_directory, Example::new())?
        .with(Processor::<Locale>::new(), "proc", &[])
        .build()?;
    game.run();
    Ok(())
}
