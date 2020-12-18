//! Example showing how to load a Locale file as an Asset using the Loader.

use amethyst::{
    assets::{AssetProcessorSystemBundle, AssetStorage, Handle, Loader, ProgressCounter},
    ecs::*,
    locale::*,
    prelude::*,
    utils::application_root_dir,
    Error,
};

struct Example {
    progress_counter: Option<ProgressCounter>,
    handle_en: Option<Handle<Locale>>,
    handle_fr: Option<Handle<Locale>>,
}

impl Example {
    pub fn new() -> Self {
        Example {
            progress_counter: None,
            handle_en: None,
            handle_fr: None,
        }
    }
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let mut progress_counter = ProgressCounter::default();

        let loader = data.resources.get::<Loader>().unwrap();

        self.handle_en = Some(loader.load(
            "locale/locale_en.ftl",
            LocaleFormat,
            &mut progress_counter,
            &data.resources.get::<AssetStorage<Locale>>().unwrap(),
        ));

        self.handle_fr = Some(loader.load(
            "locale/locale_fr.ftl",
            LocaleFormat,
            &mut progress_counter,
            &data.resources.get::<AssetStorage<Locale>>().unwrap(),
        ));

        self.progress_counter = Some(progress_counter);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        // Check if the locale has been loaded.
        if self.progress_counter.as_ref().unwrap().is_complete() {
            let store = data.resources.get::<AssetStorage<Locale>>().unwrap();

            for h in [&self.handle_en, &self.handle_fr].iter() {
                if let Some(locale) = h.as_ref().and_then(|h| store.get(h)) {
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
            Trans::Quit
        } else {
            Trans::None
        }
    }
}

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let assets_dir = application_root_dir()?.join("examples/locale/assets");

    let mut builder = DispatcherBuilder::default();

    builder.add_bundle(AssetProcessorSystemBundle::<Locale>::default());

    let mut game = Application::new(assets_dir, Example::new(), builder)?;
    game.run();
    Ok(())
}
