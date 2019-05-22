//! High level example

#![allow(unused)]

use std::sync::Arc;

use rayon::{ThreadPool, ThreadPoolBuilder};
use serde::{Deserialize, Serialize};

use amethyst_assets::*;
use amethyst_core::{
    ecs::{
        common::Errors,
        prelude::{
            Builder, Dispatcher, DispatcherBuilder, Read, ReadExpect, System, VecStorage, World,
            Write,
        },
    },
    Time,
};
use amethyst_error::{format_err, Error, ResultExt};

struct App {
    dispatcher: Dispatcher<'static, 'static>,
    state: Option<State>,
    world: World,
}

impl App {
    fn new(dispatcher: Dispatcher<'static, 'static>, path: &str, state: State) -> Self {
        let mut world = World::new();

        // Note: in an actual application, you'd want to share the thread pool.
        let pool = Arc::new(ThreadPoolBuilder::new().build().expect("Invalid config"));

        world.register::<MeshHandle>();

        world.add_resource(Errors::new());
        world.add_resource(AssetStorage::<MeshAsset>::new());
        world.add_resource(Loader::new(path, pool.clone()));
        world.add_resource(Time::default());
        world.add_resource(pool);
        world.add_resource(Time::default());

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
    const NAME: &'static str = "example::Mesh";
    type Data = VertexData;
    type HandleStorage = VecStorage<MeshHandle>;
}

/// A format the mesh data could be stored with.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Ron;

impl Format<VertexData> for Ron {
    fn name(&self) -> &'static str {
        "RON"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<VertexData, Error> {
        use ron::de::from_str;
        use std::str::from_utf8;

        let s = from_utf8(&bytes)?;

        from_str(s).with_context(|_| format_err!("Failed to decode mesh file"))
    }
}

pub struct RenderingSystem;

impl<'a> System<'a> for RenderingSystem {
    type SystemData = (
        Write<'a, AssetStorage<MeshAsset>>,
        Read<'a, Time>,
        ReadExpect<'a, Arc<ThreadPool>>,
        Option<Read<'a, HotReloadStrategy>>,
        /* texture storage, transforms, .. */
    );

    fn run(&mut self, (mut mesh_storage, time, pool, strategy): Self::SystemData) {
        use std::ops::Deref;

        let strategy = strategy.as_ref().map(Deref::deref);

        mesh_storage.process(
            |vertex_data| {
                // Upload vertex data to GPU and give back an asset

                Ok(ProcessingState::Loaded(MeshAsset { buffer: () }))
            },
            time.frame_number(),
            &**pool,
            strategy,
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
                    let a: MeshHandle =
                        loader.load("mesh.ron", Ron, &mut progress, &world.read_resource());

                    (a, progress)
                };

                world.create_entity().with(mesh).build();

                Some(State::Loading(progress))
            }
            State::Loading(progress) => match progress.complete() {
                Completion::Complete => Some(State::SomethingElse),
                Completion::Failed => {
                    eprintln!("Asset loading failed!");
                    eprintln!("-- Errors --");
                    progress.errors().iter().enumerate().for_each(|(n, e)| {
                        eprintln!("{}: error: {}", n, e.error);
                        for cause in e.error.causes().skip(1) {
                            eprintln!("{}: caused by: {}", n, cause);
                        }
                    });
                    eprintln!("Quitting game..");

                    None
                }
                Completion::Loading => Some(State::Loading(progress)),
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
        .with(RenderingSystem, "rendering", &[])
        .build();

    let assets_dir = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));
    let mut app = App::new(disp, &assets_dir, State::Start);
    app.run();
}
