extern crate amethyst;
extern crate cgmath;
extern crate obj;
extern crate yaml_rust;

use std::sync::{Mutex, Arc};

use amethyst::config::Element;
use amethyst::context::Context;
use amethyst::context::asset_manager::{Mesh, Texture};
use amethyst::context::map_manager::load_map;
use amethyst::ecs::{Join, Processor, RunArg, World};
use amethyst::engine::{Application, Config, State, Trans};
use amethyst::processors::rendering::{Camera, Light, Projection, RenderingProcessor, Renderable};
use amethyst::processors::transform::{Child, Init, LocalTransform, Transform, TransformProcessor};

struct MapsDemo;

// Set up Processor for demo
struct MapsDemoProcessor;

unsafe impl Sync for MapsDemoProcessor {  }

impl Processor<Arc<Mutex<Context>>> for MapsDemoProcessor {
    fn run(&mut self, arg: RunArg, ctx: Arc<Mutex<Context>>) {

        // Get all needed component storages and resources
        let ctx = ctx.lock().unwrap();
        let (
            mut locals,
        ) = arg.fetch(|w| (
            w.write::<LocalTransform>(),
        ));
    }
}

impl State for MapsDemo {
    fn on_start(&mut self, ctx: &mut Context, world: &mut World) {
        let (w, h) = ctx.renderer.get_dimensions().unwrap();
        let aspect = w as f32 / h as f32;

        // Get a Perspective projection
        let projection = Projection::Perspective {
            fov: 60.0,
            aspect: aspect,
            near: 1.0,
            far: 100.0,
        };

        world.add_resource::<Projection>(projection.clone());

        // Create a camera entity
        let eye =    [0., -50., 0.];
        let target = [0.,  0., 0.];
        let up =     [0.,  0., 1.];
        let mut camera = Camera::new(projection, eye, target, up);
        camera.activate();
        world.create_now()
            .with(camera)
            .build();

        // Set up world
        ctx.asset_manager.register_asset::<Mesh>();
        ctx.asset_manager.register_asset::<Texture>();

        let result = load_map::<Renderable, Light, LocalTransform, Transform>(ctx, world, "examples/08_maps/resources/maps/demo.yml");

        match result {
            Ok(_) => println!("Map successfully loaded!"),
            Err(e) => {
                panic!("Map could not be loaded: `{}`!", e.0);
            }
        }
    }

    fn update(&mut self, ctx: &mut Context, _: &mut World) -> Trans {
        // Exit if user hits Escape or closes the window
        use amethyst::context::event::{EngineEvent, Event, VirtualKeyCode};
        let engine_events = ctx.broadcaster.read::<EngineEvent>();
        for engine_event in engine_events.iter() {
            match engine_event.payload {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return Trans::Quit,
                Event::Closed => return Trans::Quit,
                _ => (),
            }
        }

        Trans::None
    }
}

fn main() {
    let path = format!("{}/examples/08_maps/resources/config.yml", env!("CARGO_MANIFEST_DIR"));
    let config = Config::from_file(path).unwrap();

    let mut ctx = Context::new(config.context_config);
    let rendering_processor = RenderingProcessor::new(config.renderer_config, &mut ctx);
    let mut game = Application::build(MapsDemo, ctx)
                   .with::<RenderingProcessor>(rendering_processor, "rendering_processor", 0)
                   .register::<Renderable>()
                   .register::<Light>()
                   .register::<Camera>()
                   .with::<MapsDemoProcessor>(MapsDemoProcessor, "maps_demo_processor", 1)
                   .with::<TransformProcessor>(TransformProcessor::new(), "transform_processor", 2)
                   .register::<LocalTransform>()
                   .register::<Transform>()
                   .register::<Child>()
                   .register::<Init>()
                   .done();
    game.run();
}
