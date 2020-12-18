//! High level example

#![allow(unused)]

use std::sync::Arc;

use amethyst_assets::*;
use amethyst_core::{ecs::*, Time};
use amethyst_error::{format_err, Error, ResultExt};
use amethyst_rendy::{Backend, Mesh};
use rayon::{ThreadPool, ThreadPoolBuilder};
use serde::{Deserialize, Serialize};
use type_uuid::*;

struct App {
    scheduler: Schedule,
    resources: Resources,
    state: Option<State>,
    world: World,
}

impl App {
    fn new(path: &str, state: State) -> Self {
        let mut world = World::default();
        let mut resources = Resources::default();

        let mut loader = DefaultLoader::default();
        loader.init_world(&mut resources);
        resources.insert(loader);

        let scheduler = Schedule::builder()
            .add_system(build_rendering_system())
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

#[derive(TypeUuid)]
#[uuid = "28d51c52-be81-4d99-8cdc-20b26eb12448"]
pub struct MeshAsset {
    /// Left out for simplicity
    /// This would for example be the gfx handle
    buffer: (),
}

impl Asset for MeshAsset {
    fn name() -> &'static str {
        "example::Mesh"
    }
    type Data = VertexData;
}

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "687b6d94-c653-4663-af73-e967c92ad140"]
pub struct VertexData {
    positions: Vec<[f32; 3]>,
    tex_coords: Vec<[f32; 2]>,
}

pub struct MeshProcessor<B: Backend> {
    marker: std::marker::PhantomData<B>,
}

impl<B: Backend> AddToDispatcher for MeshProcessor<B> {
    fn add_to_dipatcher(dispatcher_builder: &mut DispatcherBuilder) {
        ()
    }
}

amethyst_assets::register_asset_type!(VertexData => MeshAsset; MeshProcessor<amethyst_rendy::types::DefaultBackend>);

/// A format the mesh data could be stored with.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Ron;

impl Format<VertexData> for Ron {
    fn name(&self) -> &'static str {
        "RON"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<VertexData, Error> {
        use std::str::from_utf8;

        use ron::de::from_str;

        let s = from_utf8(&bytes)?;

        from_str(s).with_context(|_| format_err!("Failed to decode mesh file"))
    }
}

pub fn build_rendering_system() -> impl Runnable {
    SystemBuilder::new("RenderingSystem")
        .write_resource::<ProcessingQueue<VertexData>>()
        .write_resource::<AssetStorage<MeshAsset>>()
        .build(
            move |_commands, _world, (mesh_queue, mesh_storage), _query| {
                mesh_queue.process(mesh_storage, |vertex_data| {
                    // Upload vertex data to GPU and give back an asset
                    Ok(ProcessingState::Loaded(MeshAsset { buffer: () }))
                });
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
                        .get::<DefaultLoader>()
                        .expect("Could not get Loader resource");
                    let a: MeshHandle = loader.load("mesh.ron");

                    (a, progress)
                };

                world.push((mesh,));

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

fn main() {
    let assets_dir = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));
    let mut app = App::new(&assets_dir, State::Start);
    app.run();
}
