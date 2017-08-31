//! Shows how to use the cache.

extern crate amethyst_assets;
extern crate futures;
extern crate rayon;

use std::str::{Utf8Error, from_utf8};
use std::sync::Arc;

use amethyst_assets::*;
use rayon::{Configuration, ThreadPool};

struct DummyContext {
    cache: Cache<AssetFuture<DummyAsset>>,
    prepend: &'static str,
}

impl Context for DummyContext {
    type Asset = DummyAsset;
    type Data = String;
    type Error = NoError;
    type Result = Result<DummyAsset, NoError>;

    fn category(&self) -> &'static str {
        "dummy"
    }

    fn create_asset(&self, mut data: String, _: &ThreadPool) -> Result<DummyAsset, Self::Error> {
        data.insert_str(0, self.prepend);

        Ok(DummyAsset(Arc::new(data)))
    }

    fn cache(&self, spec: AssetSpec, asset: AssetFuture<Self::Asset>) {
        self.cache.insert(spec, asset.clone());
    }

    fn retrieve(&self, spec: &AssetSpec) -> Option<AssetFuture<Self::Asset>> {
        self.cache.get(spec)
    }

    fn clear(&self) {
        self.cache.retain(|_, _| false);
    }

    fn clear_all(&self) {
        self.cache.clear_all();
    }

    fn update(&self, _spec: &AssetSpec, _asset: AssetFuture<Self::Asset>) {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
struct DummyAsset(Arc<String>);

impl Asset for DummyAsset {
    type Context = DummyContext;
}

struct DummyFormat;

impl Format for DummyFormat {
    const EXTENSIONS: &'static [&'static str] = &["dum"];
    type Data = String;
    type Error = Utf8Error;
    type Result = Result<Self::Data, Self::Error>;

    fn parse(&self, bytes: Vec<u8>, _: &ThreadPool) -> Result<Self::Data, Self::Error> {
        from_utf8(bytes.as_slice()).map(|s| s.to_owned())
    }
}

fn main() {
    use futures::Future;

    let path = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

    let cfg = Configuration::new().num_threads(8);
    let pool = Arc::new(ThreadPool::new(cfg).expect("Invalid config"));

    let mut loader = Loader::new(&path, pool);

    loader.register(DummyContext {
        cache: Cache::new(),
        prepend: ">> ",
    });

    let dummy = loader.load("whatever", DummyFormat);
    let dummy: DummyAsset = dummy.wait().expect("Failed to load dummy asset");

    println!("dummy: {:?}", dummy);
}
