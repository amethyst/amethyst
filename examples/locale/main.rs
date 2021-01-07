//! Example showing how to load a Locale file as an Asset using the Loader.

use amethyst::{
    assets::{AssetStorage, DefaultLoader, Handle, Loader, LoaderBundle},
    ecs::*,
    locale::*,
    prelude::*,
    renderer::{types::DefaultBackend, RenderingBundle},
    utils::application_root_dir,
    Error,
};

struct Example {
    handle_en: Option<Handle<Locale>>,
    handle_fr: Option<Handle<Locale>>,
}

impl Example {
    pub fn new() -> Self {
        Example {
            handle_en: None,
            handle_fr: None,
        }
    }
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let loader = data.resources.get::<DefaultLoader>().unwrap();
        self.handle_en = Some(loader.load("locale/locale_en.ftl"));
        self.handle_fr = Some(loader.load("locale/locale_fr.ftl"));
    }

    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        let _loader = data.resources.get::<DefaultLoader>().unwrap();

        // Check if the locale has been loaded.
        let store = data.resources.get::<AssetStorage<Locale>>().unwrap();

        for h in [self.handle_en.as_ref(), self.handle_fr.as_ref()].iter() {
            if let Some(locale) = h.and_then(|h| store.get(h)) {
                let bundle = &locale.bundle;
                let msg_hello = bundle
                    .get_message("hello")
                    .expect("Failed to load message for hello");
                let msg_bye = bundle
                    .get_message("bye")
                    .expect("Failed to load message for bye");
                let hello_value = msg_hello.value.expect("Hello message has no value");
                let bye_value = msg_bye.value.expect("Bye message has no value");

                let mut errors = vec![];
                println!("{}", bundle.format_pattern(hello_value, None, &mut errors));
                println!("{}", bundle.format_pattern(bye_value, None, &mut errors));
                assert_eq!(errors.len(), 0);
            }
        }

        Trans::None
    }
}

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let assets_dir = app_root.join("assets");

    let mut builder = DispatcherBuilder::default();

    builder
        .add_bundle(LoaderBundle)
        .add_bundle(RenderingBundle::<DefaultBackend>::new());

    let game = Application::new(assets_dir, Example::new(), builder)?;
    game.run();
    Ok(())
}
