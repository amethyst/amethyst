//! High level example

#![allow(unused)]

extern crate amethyst_assets;
extern crate rayon;
extern crate ron;
#[macro_use]
extern crate serde;
extern crate specs;

use std::sync::Arc;

use amethyst_assets::*;
use rayon::ThreadPool;
use specs::{DenseVecStorage, Dispatcher, DispatcherBuilder, Fetch, FetchMut, System, World};
use specs::common::Errors;

struct App {
    dispatcher: Dispatcher<'static, 'static>,
    state: Option<State>,
    world: World,
}

impl App {
    fn new(dispatcher: Dispatcher<'static, 'static>, path: &str, state: State) -> Self {
        let mut world = World::new();

        // Note: in an actual application, you'd want to share the thread pool.
        let pool = Arc::new(ThreadPool::new(Default::default()).expect("Invalid config"));

        world.register::<MeshHandle>();

        world.add_resource(Errors::new());
        world.add_resource(AssetStorage::<MeshAsset>::new());
        world.add_resource(Loader::new(path, pool));

        App {
            dispatcher,
            state: Some(state),
            world,
        }
    }

    fn update(&mut self) {
        self.dispatcher.dispatch(&mut self.world.res);
        self.world.maintain();
        let mut errors = self.world.write_resource::<Errors>();
        errors.print_and_exit();
    }

    fn run(&mut self) {
        loop {
            self.update();
            match self.state.take().unwrap().update(&mut self.world) {
                Some(state) => self.state = Some(state),
                None => return,
            }
        }
    }
}

type MeshHandle = Handle<MeshAsset>;

pub struct MeshAsset {
    /// Left out for simplicity
    /// This would for example be the gfx handle
    buffer: (),
}

impl Asset for MeshAsset {
    type Data = VertexData;
    type HandleStorage = DenseVecStorage<MeshHandle>;
}

/// A format the mesh data could be stored with.
#[derive(Clone)]
struct Ron;

impl SimpleFormat<MeshAsset> for Ron {
    const NAME: &'static str = "RON";

    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<VertexData, BoxedErr> {
        use ron::de::from_str;
        use std::str::from_utf8;

        let s = from_utf8(&bytes).map_err(BoxedErr::new)?;

        from_str(s).map_err(BoxedErr::new)
    }
}

pub struct RenderingSystem;

impl<'a> System<'a> for RenderingSystem {
    type SystemData = (
        FetchMut<'a, AssetStorage<MeshAsset>>,
        Fetch<'a, Errors>,
                       /* texture storage, transforms, .. */
    );

    fn run(&mut self, (mut mesh_storage, errors): Self::SystemData) {
        mesh_storage.process(
            |vertex_data| {
                // Upload vertex data to GPU and give back an asset

                Ok(MeshAsset { buffer: () })
            },
            &errors,
        );
    }
}

enum State {
    Start,
    Loading(ProgressCounter),
    SomethingElse,
}

impl State {
    /// Returns `Some` if the app should quit.
    fn update(self, world: &mut World) -> Option<Self> {
        match self {
            State::Start => {
                let (mesh, progress) = {
                    let mut progress = ProgressCounter::new();
                    let loader = world.read_resource::<Loader>();
                    let a = loader.load("mesh.ron", Ron, (), &mut progress, &world.read_resource());

                    (a, progress)
                };

                world.create_entity().with(mesh).build();

                Some(State::Loading(progress))
            }
            State::Loading(progress) => if progress.is_complete() {
                Some(State::SomethingElse)
            } else {
                Some(State::Loading(progress))
            },
            State::SomethingElse => {
                // You could now start the actual game, cause the loading is done.
                // This example however will just quit.

                println!("Mesh is loaded and the game can begin!");
                println!("Game ending, sorry");

                None
            }
        }
    }
}

#[derive(Deserialize)]
pub struct VertexData {
    positions: Vec<[f32; 3]>,
    tex_coords: Vec<[f32; 2]>,
}

fn main() {
    let disp = DispatcherBuilder::new()
        .add(RenderingSystem, "rendering", &[])
        .build();

    let mut app = App::new(disp, "examples/assets/", State::Start);
    app.run();
}
