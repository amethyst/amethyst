//! Shows how to use the cache.

extern crate amethyst_assets;
extern crate futures;
extern crate rayon;

use std::str::{Utf8Error, from_utf8};
use std::sync::Arc;

use amethyst_assets::*;
use rayon::{Configuration, ThreadPool};

struct Context {
    cache: Cache<DummyAsset>,
    prepend: &'static str,
}

#[derive(Clone, Debug)]
struct DummyAsset(Arc<String>);

impl Asset for DummyAsset {
    type Context = Context;
    type Data = String;
    type Error = NoError;

    fn category() -> &'static str {
        "dummy"
    }

    fn from_data(mut data: String, context: &Context) -> Result<Self, Self::Error> {
        data.insert_str(0, &context.prepend);

        Ok(DummyAsset(Arc::new(data)))
    }

    fn cache(context: &Context, spec: AssetSpec, asset: &Self) {
        context.cache.insert(spec, asset.clone());
    }

    fn retrieve(context: &Context, spec: &AssetSpec) -> Option<Self> {
        context.cache.get(spec)
    }

    fn clear(context: &Context) {
        context.cache.retain(|_, a| Arc::strong_count(&a.0) > 1);
    }

    fn clear_all(context: &Context) {
        context.cache.clear_all();
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

    loader.register::<DummyAsset>(Context {
                                      cache: Cache::new(),
                                      prepend: ">> ",
                                  });

    let dummy = loader.load("whatever", DummyFormat);
    let dummy: DummyAsset = dummy.wait().expect("Failed to load dummy asset");

    println!("dummy: {:?}", dummy);
}
