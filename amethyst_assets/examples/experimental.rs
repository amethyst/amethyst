//! High level example

#![allow(unused)]

use std::{path::PathBuf, sync::Arc};

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
use type_uuid::*;

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
    type HandleStorage = VecStorage<Handle<MeshAsset>>;
}

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "687b6d94-c653-4663-af73-e967c92ad140"]
pub struct VertexData {
    positions: Vec<[f32; 3]>,
    tex_coords: Vec<[f32; 2]>,
}
/// A format the mesh data could be stored with.
#[derive(Debug, Default, Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "df3c6c87-05e6-4cc9-8711-cb6a6aad9942"]
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
amethyst_assets::register_asset_type!(VertexData => MeshAsset);
amethyst_assets::register_importer!(".ron", Ron);

struct App {
    dispatcher: Dispatcher<'static, 'static>,
    state: Option<State>,
    world: World,
}

impl App {
    fn new(dispatcher: Dispatcher<'static, 'static>, state: State) -> Self {
        let mut world = World::new();

        world.add_resource(Errors::new());
        world.add_resource(Time::default());
        let mut loader = NewDefaultLoader::default();
        loader.init_world(&mut world.res);
        world.add_resource(loader);

        App {
            dispatcher,
            state: Some(state),
            world,
        }
    }

    fn update(&mut self) {
        self.dispatcher.dispatch(&mut self.world.res);
        self.world.maintain();
        let mut loader = self.world.write_resource::<NewDefaultLoader>();
        loader.process(&self.world.res);
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
pub struct RenderingSystem;

impl<'a> System<'a> for RenderingSystem {
    type SystemData = (
        Read<'a, NewDefaultLoader>,
        Read<'a, Time>,
        Write<'a, ProcessingQueue<VertexData>>,
        Write<'a, NewAssetStorage<MeshAsset>>,
    );

    fn run(&mut self, (loader, time, mut processing_queue, mut storage): Self::SystemData) {
        processing_queue.process(&mut *storage, |vertex_data| {
            Ok(NewProcessingState::Loaded(MeshAsset { buffer: () }))
        });
    }
}

enum State {
    Start,
    Loading(GenericHandle),
    SomethingElse,
}

impl State {
    /// Returns `Some` if the app should quit.
    fn update(self, world: &mut World) -> Option<Self> {
        match self {
            State::Start => {
                let loader = world.read_resource::<NewDefaultLoader>();
                Some(State::Loading(
                    loader.load_asset_generic(
                        *uuid::Uuid::parse_str("39c7043a-dd7e-4654-9b22-e45d5c6b87cc")
                            .unwrap()
                            .as_bytes(),
                    ),
                ))
            }
            State::Loading(handle) => {
                let loader = world.read_resource::<NewDefaultLoader>();
                match handle.get_load_status(&*loader) {
                    LoadStatus::Loaded => Some(State::SomethingElse),
                    _ => Some(State::Loading(handle)),
                }
            }
            State::SomethingElse => {
                // You could now start the actual game, cause the loading is done.
                // This example however will just quit.

                println!("Asset is loaded and the game can begin!");
                println!("Game ending, sorry");
                None
            }
        }
    }
}

fn main() {
    let examples_dir = PathBuf::from(format!("{}/examples", env!("CARGO_MANIFEST_DIR")));
    let assets_dir = examples_dir.join("assets");
    atelier_daemon::init_logging();

    // launch an asset daemon in a separate thread
    std::thread::spawn(move || {
        atelier_daemon::AssetDaemon::default()
            .with_importers(
                atelier_importer::get_source_importers().map(|i| (i.extension, (i.instantiator)())),
            )
            .with_asset_dirs(vec![assets_dir])
            .with_db_path(examples_dir.join(".asset_db"))
            .run();
    });

    let disp = DispatcherBuilder::new()
        .with(RenderingSystem, "rendering", &[])
        .build();

    let mut app = App::new(disp, State::Start);
    app.run();
}
