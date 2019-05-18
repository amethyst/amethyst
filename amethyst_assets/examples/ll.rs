//! Defining a custom asset and format.

use std::{str::from_utf8, sync::Arc, thread::sleep, time::Duration};

use rayon::ThreadPoolBuilder;

use amethyst_assets::*;
use amethyst_core::ecs::prelude::VecStorage;
use amethyst_error::Error;

#[derive(Clone, Debug)]
struct DummyAsset(String);

impl Asset for DummyAsset {
    const NAME: &'static str = "example::DummyAsset";
    type Data = String;
    type HandleStorage = VecStorage<Handle<DummyAsset>>;
}

#[derive(Clone, Debug)]
struct DummyFormat;

impl Format<String> for DummyFormat {
    fn name(&self) -> &'static str {
        "DUMMY"
    }

    fn import(
        &self,
        name: String,
        source: Arc<dyn Source>,
        _create_reload: Option<Box<dyn Format<String>>>,
    ) -> Result<FormatValue<String>, Error> {
        let dummy = from_utf8(source.load(&name)?.as_slice()).map(|s| s.to_owned())?;

        Ok(FormatValue::data(dummy))
    }
}

fn main() {
    let path = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

    let builder = ThreadPoolBuilder::new().num_threads(8);
    let pool = Arc::new(builder.build().expect("Invalid config"));

    let loader = Loader::new(&path, pool.clone());
    let mut storage: AssetStorage<DummyAsset> = AssetStorage::new();

    let mut progress = ProgressCounter::new();

    let dummy = loader.load("dummy/whatever.dum", DummyFormat, &mut progress, &storage);

    // Hot-reload every three seconds.
    let strategy = HotReloadStrategy::every(3);

    // Game loop
    let mut frame_number = 0;

    loop {
        frame_number += 1;

        // If loading is done, end the game loop and print the asset
        if progress.is_complete() {
            break;
        }

        // Do per-frame stuff (display loading screen, ..)
        sleep(Duration::new(1, 0));

        storage.process(
            |mut s| {
                s.insert_str(0, ">> ");

                Ok(ProcessingState::Loaded(DummyAsset(s)))
            },
            frame_number,
            &*pool,
            Some(&strategy),
        );
    }

    println!("dummy: {:?}", storage.get(&dummy));
}
