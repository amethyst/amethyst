//! Displays a multicolored sphere to the user.
//!
//! TODO: Rewrite for new renderer.

extern crate amethyst;
extern crate amethyst_renderer;
extern crate cgmath;
extern crate genmesh;
extern crate winit;

use amethyst::prelude::*;
use amethyst::ecs::components::*;
use amethyst::renderer::prelude::*;
use amethyst::renderer::Config as DisplayConfig;

use cgmath::{Deg, Vector3};
use cgmath::prelude::InnerSpace;
use genmesh::{MapToVertices, Triangulate, Vertices};
use genmesh::generators::SphereUV;

struct Example;

impl State for Example {
    fn on_start(&mut self, engine: &mut Engine) {
        let verts = gen_sphere(32, 32);
        let mesh = Mesh::build(verts);
        let tex = Texture::from_color_val([0.0, 0.0, 1.0, 1.0]);
        let mtl = MaterialBuilder::new().with_albedo(tex);

        engine.world.create_entity()
            .with(Transform::default())
            //.with(mesh.unfinished()) // FIXME: asset loader pending
            //.with(mtl.unfinished()) // FIXME: asset loader pending
            .build();

        engine.world.create_entity()
            .with(LightComponent(PointLight {
                center: [2.0, 2.0, 2.0].into(),
                radius: 5.0,
                intensity: 3.0,
                ..Default::default()
            }.into()))
            .build();

        engine.world.add_resource(Camera {
            eye: [0.0, 0.0, -4.0].into(),
            proj: Projection::perspective(1.3, Deg(60.0)).into(),
            forward: [0.0, 0.0, 1.0].into(),
            right: [1.0, 0.0, 0.0].into(),
            up: [0.0, 1.0, 0.0].into(),
        });
    }

    fn handle_event(&mut self, _: &mut Engine, event: Event) -> Trans {
        match event {
            Event::WindowEvent { event, .. } => match event {
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

fn main() {
    let path = format!("{}/examples/02_sphere/resources/config.ron",
                       env!("CARGO_MANIFEST_DIR"));
    let config = DisplayConfig::load(&path);
    let builder = Application::build(Example)
        .with_renderer(Pipeline::build()
            .with_stage(Stage::with_backbuffer()
                .clear_target([0.00196, 0.23726, 0.21765, 1.0], 1.0)
                .with_model_pass(pass::DrawFlat::<PosNormTex>::new())
            ),
            config)
    .unwrap();

    let mut game = builder
        .build()
        .expect("Fatal error");
    game.run();
}

fn gen_sphere(u: usize, v: usize) -> Vec<PosNormTex> {
    SphereUV::new(u, v)
        .vertex(|(x, y, z)| {
            PosNormTex {
                a_position: [x, y, z],
                a_normal: Vector3::from([x, y, z]).normalize().into(),
                a_tex_coord: [0.1, 0.1],
            }
        })
        .triangulate()
        .vertices()
        .collect()
}
