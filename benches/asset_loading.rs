#![feature(test)]

extern crate amethyst;
extern crate test;

use amethyst::Context;
use amethyst::asset_manager::{AssetLoader, DirectoryStore};
use amethyst::asset_manager::formats::{Obj, Png};
use amethyst::ecs::components::{Mesh, Texture};

use test::black_box;
use test::Bencher;

fn create_context() -> Context {
    use amethyst::gfx_device::video_init;

    let (_, factory, _) = video_init(Default::default());
    Context::new(factory)
}

#[bench]
fn bench_parallel(b: &mut Bencher) {
    let mut context = create_context();
    let store = DirectoryStore::new(&format!("{}/benches/assets", env!("CARGO_MANIFEST_DIR")));
    let loader = AssetLoader::new();

    b.iter(|| {
        let a = loader.load(&store, "cone", Obj);
        let b = loader.load(&store, "cone", Obj);
        let c = loader.load(&store, "cone", Obj);
        let d = loader.load(&store, "cone", Obj);

        let tex1 = Texture::from_color([1.0, 0.0, 1.0, 1.0]);
        let tex2 = loader.load(&store, "grass", Png);

        let context = &mut context;
        let a: Mesh = a.finish(context).unwrap();
        let b: Mesh = b.finish(context).unwrap();
        let c: Mesh = c.finish(context).unwrap();
        let d: Mesh = d.finish(context).unwrap();

        let tex2: Texture = tex2.finish(context).unwrap();

        black_box(a);
        black_box(b);
        black_box(c);
        black_box(d);
        black_box(tex1);
        black_box(tex2);
    });
}

#[bench]
fn bench_single_threaded(b: &mut Bencher) {
    let mut context = create_context();
    let store = DirectoryStore::new(&format!("{}/benches/assets", env!("CARGO_MANIFEST_DIR")));

    b.iter(|| {
        let context = &mut context;

        let a: Mesh = AssetLoader::load_now(&store, "cone", Obj, context).unwrap();
        let b: Mesh = AssetLoader::load_now(&store, "cone", Obj, context).unwrap();
        let c: Mesh = AssetLoader::load_now(&store, "cone", Obj, context).unwrap();
        let d: Mesh = AssetLoader::load_now(&store, "cone", Obj, context).unwrap();

        let tex1 = Texture::from_color([1.0, 0.0, 1.0, 1.0]);
        let tex2: Texture = AssetLoader::load_now(&store, "grass", Png, context).unwrap();

        black_box(a);
        black_box(b);
        black_box(c);
        black_box(d);
        black_box(tex1);
        black_box(tex2);
    });
}
