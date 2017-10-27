//! Defining a custom asset and format.

extern crate amethyst_assets;
extern crate rayon;
extern crate specs;

use std::str::from_utf8;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use amethyst_assets::*;
use rayon::{Configuration, ThreadPool};
use specs::DenseVecStorage;

#[derive(Clone, Debug)]
struct DummyAsset(String);

impl Asset for DummyAsset {
    type Data = String;
    type HandleStorage = DenseVecStorage<Handle<DummyAsset>>;
}

struct DummyFormat;

impl Format<DummyAsset> for DummyFormat {
    const NAME: &'static str = "DUMMY";

    type Options = ();

    fn import(
        &self,
        name: String,
        source: Arc<Source>,
        _: (),
        _create_reload: bool,
    ) -> Result<FormatValue<DummyAsset>> {
        let dummy = from_utf8(source.load(&name)?.as_slice())
            .map(|s| s.to_owned())
            .chain_err(|| "Bytes are invalid UTF-8")?;

        Ok(FormatValue::data(dummy))
    }
}

fn main() {
    let path = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

    let cfg = Configuration::new().num_threads(8);
    let pool = Arc::new(ThreadPool::new(cfg).expect("Invalid config"));

    let loader = Loader::new(&path, pool.clone());
    let mut storage = AssetStorage::new();

    let mut progress = ProgressCounter::new();

    let dummy = loader.load(
        "dummy/whatever.dum",
        DummyFormat,
        (),
        &mut progress,
        &storage,
    );

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

                Ok(DummyAsset(s))
            },
            frame_number,
            &*pool,
            Some(&strategy),
        );
    }

    println!("dummy: {:?}", storage.get(&dummy));
}
