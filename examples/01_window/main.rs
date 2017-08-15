//! Opens an empty window.

extern crate amethyst;
extern crate amethyst_renderer;

use amethyst::prelude::*;
use amethyst::ecs::World;
use amethyst::ecs::systems::RenderSystem;
use amethyst::event::{KeyboardInput, VirtualKeyCode};

use amethyst_renderer::prelude::*;

struct Example;

impl State for Example {
    fn handle_event(&mut self, _: &mut Engine, event: Event) -> Trans {
        match event {
            Event::WindowEvent {
                event, ..
            } => match event {
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Escape), ..
                    }, ..
                } | WindowEvent::Closed => Trans::Quit,
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }
}

fn register(world: &mut World) {
    use amethyst::ecs::components::*;

    world.register::<Transform>();
    world.register::<MeshComponent>();
    world.register::<MaterialComponent>();
    world.register::<LightComponent>();
    world.register::<Unfinished<MeshComponent>>();
    world.register::<Unfinished<MaterialComponent>>();
}

fn main() {

    let path = format!("{}/examples/01_window/resources/config.ron",
                       env!("CARGO_MANIFEST_DIR"));

    let builder = Application::build(Example);
    let render = RenderSystem::new(
        &builder.events,
        Pipeline::forward::<PosNormTex>()
    ).unwrap();

    let mut game = builder
        .with_thread_local(render)
        .build()
        .expect("Fatal error");

    register(&mut game.engine.world);

    game.run();
}
