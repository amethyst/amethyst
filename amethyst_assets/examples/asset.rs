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
    type Data = String;
    type Error = NoError;

    fn is_shared(&self) -> bool {
        false
    }

    fn push_update(&self, _updated: Self) {
        unimplemented!()
    }

    fn update(&mut self) {
        unimplemented!()
    }
}

struct DummyContext(&'static str);

impl Context for DummyContext {
    type Asset = DummyAsset;
    type Data = String;
    type Error = NoError;

    fn category(&self) -> &str {
        "dummy"
    }

    fn create_asset(&self, mut data: String) -> Result<DummyAsset, Self::Error> {
        data.insert_str(0, self.0);

        Ok(DummyAsset(data))
    }

    fn update(&self, _spec: &AssetSpec, _asset: Self::Asset) {
        unimplemented!()
    }
}

struct DummyFormat;

impl Format for DummyFormat {
    type Data = String;
    type Error = Utf8Error;

    fn extension() -> &'static str {
        "dum"
    }

    fn parse(&self, bytes: Vec<u8>) -> Result<Self::Data, Self::Error> {
        from_utf8(bytes.as_slice()).map(|s| s.to_owned())
    }
}

fn main() {
    use futures::Future;

    let path = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

    let cfg = Configuration::new().num_threads(8);
    let pool = Arc::new(ThreadPool::new(cfg).expect("Invalid config"));

    let alloc = Allocator::new();
    let mut loader = Loader::new(&alloc, &path, pool);

    loader.register(DummyContext(">> "));

    let dummy = loader.load("whatever", DummyFormat);
    let dummy: DummyAsset = dummy.wait().expect("Failed to load dummy asset");

    println!("dummy: {:?}", dummy);
}
