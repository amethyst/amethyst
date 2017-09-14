//! Defining a custom asset and format.

extern crate amethyst_assets;
extern crate futures;
extern crate rayon;

use std::str::{Utf8Error, from_utf8};
use std::sync::Arc;

use amethyst_assets::*;
use rayon::{Configuration, ThreadPool};

#[derive(Clone, Debug)]
struct DummyAsset(String);

impl Asset for DummyAsset {
    type Context = DummyContext;
}

struct DummyContext(&'static str);

impl Context for DummyContext {
    type Result = Result<DummyAsset, NoError>;
    type Asset = DummyAsset;
    type Data = String;
    type Error = NoError;

    fn category(&self) -> &str {
        "dummy"
    }

    fn create_asset(&self, mut data: String, _: &ThreadPool) -> Result<DummyAsset, Self::Error> {
        data.insert_str(0, self.0);

        Ok(DummyAsset(data))
    }

    fn update(&self, _spec: &AssetSpec, _asset: AssetFuture<Self::Asset>) {
        unimplemented!()
    }
}

struct DummyFormat;

impl Format for DummyFormat {
    const EXTENSIONS: &'static [&'static str] = &["dum"];
    type Result = Result<String, Utf8Error>;
    type Data = String;
    type Error = Utf8Error;

    fn parse(&self, bytes: Vec<u8>, _: &ThreadPool) -> Self::Result {
        from_utf8(bytes.as_slice()).map(|s| s.to_owned())
    }
}

fn main() {
    use futures::Future;

    let path = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

    let cfg = Configuration::new().num_threads(8);
    let pool = Arc::new(ThreadPool::new(cfg).expect("Invalid config"));

    let mut loader = Loader::new(&path, pool);

    loader.register(DummyContext(">> "));

    let dummy = loader.load("whatever", DummyFormat);
    let dummy: DummyAsset = dummy.wait().expect("Failed to load dummy asset");

    println!("dummy: {:?}", dummy);
}
