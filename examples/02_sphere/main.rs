//! Displays a multicolored sphere to the user.

extern crate amethyst;
extern crate cgmath;

use amethyst::engine::{Application, State, Trans};
use amethyst::config::Element;
use amethyst::ecs::World;
use amethyst::gfx_device::DisplayConfig;
use amethyst::asset_manager::AssetManager;
use amethyst::context::event::EngineEvent;

struct Example;

impl State for Example {
    fn on_start(&mut self, _: &mut World, _: &mut AssetManager) {
        use amethyst::renderer::pass::{Clear, DrawShaded};
        use amethyst::renderer::{Layer, Camera, Light};
        use cgmath::Vector3;

        let (w, h) = ctx.renderer.get_dimensions().unwrap();
        let proj = Camera::perspective(60.0, w as f32 / h as f32, 1.0, 100.0);
        let eye = [0.0, 5.0, 0.0];
        let target = [0.0, 0.0, 0.0];
        let up = [0.0, 0.0, 1.0];
        let view = Camera::look_at(eye, target, up);
        let camera = Camera::new(proj, view);

        ctx.renderer.add_scene("main");
        ctx.renderer.add_camera(camera, "main");

        ctx.asset_manager.register_asset::<Mesh>();
        ctx.asset_manager.register_asset::<Texture>();
        ctx.asset_manager.create_constant_texture("dark_blue", [0.0, 0.0, 0.01, 1.0]);
        ctx.asset_manager.create_constant_texture("green", [0.0, 1.0, 0.0, 1.0]);
        ctx.asset_manager.gen_sphere("sphere", 32, 32);

        let translation = Vector3::new(0.0, 0.0, 0.0);
        let transform: [[f32; 4]; 4] = cgmath::Matrix4::from_translation(translation).into();
        let fragment = ctx.asset_manager.get_fragment("sphere", "dark_blue", "green", transform).unwrap();

        ctx.renderer.add_fragment("main", fragment);

        let light = Light {
            color: [1.0, 1.0, 1.0, 1.0],
            radius: 1.0,
            center: [2.0, 2.0, 2.0],
            propagation_constant: 0.0,
            propagation_linear: 0.0,
            propagation_r_square: 1.0,
        };

        ctx.renderer.add_light("main", light);

        let layer =
            Layer::new("main",
                        vec![
                            Clear::new([0.0, 0.0, 0.0, 1.0]),
                            DrawShaded::new("main", "main"),
                        ]);

        let pipeline = vec![layer];
        ctx.renderer.set_pipeline(pipeline);
    }

    fn handle_events(&mut self, events: &[EngineEvent], _: &mut World, _: &mut AssetManager) -> Trans {
        use amethyst::context::event::*;
        for event in events {
            match event.payload {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return Trans::Quit,
                Event::Closed => return Trans::Quit,
                _ => (),
            }
        }
        Trans::None
    }
}

fn main() {
    let path = format!("{}/examples/02_sphere/resources/config.yml",
                       env!("CARGO_MANIFEST_DIR"));
    let config = ContextConfig::from_file(path).unwrap();
    let ctx = Context::new(config);
    let mut game = Application::build(Example, ctx).done();
    game.run();
}

// /// Generate and load a sphere mesh using the number of vertices accross the equator (u)
// /// and the number of vertices from pole to pole (v).
// fn gen_sphere(&mut self, name: &str, u: usize, v: usize) -> Vec<VertexPosNormal> {
//     let data: Vec<VertexPosNormal> = SphereUV::new(u, v)
//         .vertex(|(x, y, z)| {
//             VertexPosNormal {
//                 pos: [x, y, z],
//                 normal: Vector3::new(x, y, z).normalize().into(),
//                 tex_coord: [0., 0.],
//             }
//         })
//         .triangulate()
//         .vertices()
//         .collect()
// }
