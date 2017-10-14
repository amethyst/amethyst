//! Defining a custom asset and format.

extern crate amethyst_assets;
extern crate rayon;
extern crate specs;

use std::str::from_utf8;
use std::sync::Arc;

use amethyst_assets::*;
use rayon::{Configuration, ThreadPool};
use specs::common::Errors;

#[derive(Clone, Debug)]
struct DummyAsset(String);

impl Asset for DummyAsset {
    type Data = String;
}

struct DummyFormat;

impl Format<DummyAsset> for DummyFormat {
    const NAME: &'static str = "DUMMY";

    fn import(&self, name: String, source: Arc<Source>) -> Result<String, BoxedErr> {
        from_utf8(source.load(&name)?.as_slice())
            .map(|s| s.to_owned())
            .map_err(BoxedErr::new)
    }
}

fn main() {
    let path = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

    let cfg = Configuration::new().num_threads(8);
    let pool = Arc::new(ThreadPool::new(cfg).expect("Invalid config"));

    let mut errors = Errors::new();
    let loader = Loader::new(&path, pool);
    let mut storage = AssetStorage::new();

    let dummy = loader.load("dummy/whatever.dum", DummyFormat, &storage);

    loop {
        if storage.get(&dummy).is_some() {
            break;
        }

        use std::thread::sleep_ms;
        sleep_ms(100);

        storage.process(
            |mut s| {
                s.insert_str(0, ">> ");

                Ok(DummyAsset(s))
            },
            &errors,
        );

        errors.print_and_exit();
    }

    println!("dummy: {:?}", storage.get(&dummy));
}
