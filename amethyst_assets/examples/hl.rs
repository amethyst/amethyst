//! High level example

#![allow(unused)]

use std::sync::Arc;

use rayon::{ThreadPool, ThreadPoolBuilder};
use serde::{Deserialize, Serialize};

use amethyst_assets::*;
use amethyst_core::{
    dispatcher::{DispatcherBuilder, Stage},
    ecs::prelude::*,
    Time,
};
use amethyst_error::{format_err, Error, ResultExt};

struct App {
    scheduler: Schedule,
    resources: Resources,
    state: Option<State>,
    world: World,
}

impl App {
    fn new(path: &str, state: State) -> Self {
        let mut world = World::new();

        // Note: in an actual application, you'd want to share the thread pool.
        let pool = Arc::new(ThreadPoolBuilder::new().build().expect("Invalid config"));
        let mut resources = Resources::default();

        resources.insert(AssetStorage::<MeshAsset>::new());
        resources.insert(Loader::new(path, pool.clone()));
        resources.insert(Time::default());
        resources.insert(pool);
        resources.insert(Time::default());
        resources.insert::<Option<HotReloadStrategy>>(None);

        let scheduler = Schedule::builder()
            .add_system(build_rendering_system(&mut world, &mut resources))
            .build();

        App {
            scheduler,
            resources,
            state: Some(state),
            world,
        }
    }

    fn update(&mut self) {
        self.scheduler.execute(&mut self.world, &mut self.resources);
    }

    fn run(&mut self) {
        loop {
            self.update();
            match self
                .state
                .take()
                .unwrap()
                .update(&mut self.world, &mut self.resources)
            {
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

pub fn build_rendering_system(
    world: &mut World,
    resources: &mut Resources,
) -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("RenderingSystem")
        .read_resource::<Time>()
        .read_resource::<Arc<ThreadPool>>()
        .read_resource::<Option<HotReloadStrategy>>()
        .write_resource::<AssetStorage<MeshAsset>>()
        .build(
            move |_commands, _world, (time, pool, strategy, mesh_storage), _query| {
                use std::ops::Deref;
                let strategy = strategy.as_ref();

                mesh_storage.process(
                    |vertex_data| {
                        // Upload vertex data to GPU and give back an asset

                        Ok(ProcessingState::Loaded(MeshAsset { buffer: () }))
                    },
                    time.frame_number(),
                    &**pool,
                    strategy,
                );
            },
        )
}

enum State {
    Start,
    Loading(ProgressCounter),
    SomethingElse,
}

impl State {
    /// Returns `Some` if the app should quit.
    fn update(self, world: &mut World, resources: &mut Resources) -> Option<Self> {
        match self {
            State::Start => {
                let (mesh, progress) = {
                    let mut progress = ProgressCounter::new();
                    let loader = resources
                        .get::<Loader>()
                        .expect("Could not get Loader resource");
                    let a: MeshHandle =
                        loader.load("mesh.ron", Ron, &mut progress, &resources.get().unwrap());

                    (a, progress)
                };

                world.insert((), vec![(mesh,)]);

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
    let assets_dir = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));
    let mut app = App::new(&assets_dir, State::Start);
    app.run();
}
