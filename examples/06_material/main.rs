//! Displays spheres with physically based materials.

extern crate amethyst;
extern crate cgmath;
extern crate futures;
extern crate genmesh;

use amethyst::assets::{AssetFuture, BoxedErr};
use amethyst::ecs::rendering::{LightComponent, MaterialComponent, AmbientColor, Factory,
                               MeshComponent, RenderBundle};
use amethyst::ecs::transform::Transform;
use amethyst::prelude::*;
use amethyst::renderer::Config as DisplayConfig;
use amethyst::renderer::prelude::*;
use cgmath::{Deg, Matrix4, Vector3};
use cgmath::prelude::InnerSpace;
use futures::{Future, IntoFuture};
use genmesh::{MapToVertices, Triangulate, Vertices};
use genmesh::generators::SphereUV;

struct Example;



fn load_proc_asset<T, F>(engine: &mut Engine, f: F) -> AssetFuture<T::Item>
where
    T: IntoFuture<Error = BoxedErr>,
    T::Future: 'static,
    F: FnOnce(&mut Engine) -> T,
{
    let future = f(engine).into_future();
    let future: Box<Future<Item = T::Item, Error = BoxedErr>> = Box::new(future);
    AssetFuture(future.shared())
}

impl State for Example {
    fn on_start(&mut self, engine: &mut Engine) {
        let verts = gen_sphere(32, 32);
        let mesh = Mesh::build(verts);
        let tex = Texture::from_color_val([1.0, 1.0, 1.0, 1.0]);
        let mtl = MaterialBuilder::new().with_albedo(tex);

        println!("Load mesh");
        let mesh = load_proc_asset(engine, move |engine| {
            let factory = engine.world.read_resource::<Factory>();
            factory.create_mesh(mesh).map(MeshComponent::new).map_err(
                BoxedErr::new,
            )
        });


        println!("Create spheres");
        for i in 0..5 {
            for j in 0..5 {
                let roughness = 1.0f32 * (i as f32 / 4.0f32);
                let metallic = 1.0f32 * (j as f32 / 4.0f32);
                let pos = Matrix4::from_translation(
                    [2.0f32 * (i - 2) as f32, 2.0f32 * (j - 2) as f32, 0.0].into(),
                );

                let metallic = Texture::from_color_val([metallic, metallic, metallic, 1.0]);
                let roughness = Texture::from_color_val([roughness, roughness, roughness, 1.0]);

                let mtl = mtl.clone().with_metallic(metallic).with_roughness(
                    roughness,
                );

                let mtl = load_proc_asset(engine, move |engine| {
                    let factory = engine.world.read_resource::<Factory>();
                    factory
                        .create_material(mtl)
                        .map(MaterialComponent)
                        .map_err(BoxedErr::new)
                });
                engine
                    .world
                    .create_entity()
                    .with(Transform(pos.into()))
                    .with(mesh.clone())
                    .with(mtl)
                    .build();
            }
        }


        println!("Create lights");
        engine
            .world
            .create_entity()
            .with(LightComponent(
                PointLight {
                    center: [6.0, 6.0, -6.0].into(),
                    intensity: 6.0,
                    color: [0.8, 0.0, 0.0].into(),
                    ..PointLight::default()
                }.into(),
            ))
            .build();

        engine
            .world
            .create_entity()
            .with(LightComponent(
                PointLight {
                    center: [6.0, -6.0, -6.0].into(),
                    intensity: 5.0,
                    color: [0.0, 0.3, 0.7].into(),
                    ..PointLight::default()
                }.into(),
            ))
            .build();

        println!("Put camera");
        engine.world.add_resource(Camera {
            eye: [0.0, 0.0, -12.0].into(),
            proj: Projection::perspective(1.3, Deg(60.0)).into(),
            forward: [0.0, 0.0, 1.0].into(),
            right: [1.0, 0.0, 0.0].into(),
            up: [0.0, 1.0, 0.0].into(),
        });
    }

    fn handle_event(&mut self, _: &mut Engine, event: Event) -> Trans {
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Escape), .. }, ..
                    } |
                    WindowEvent::Closed => Trans::Quit,
                    _ => Trans::None,
                }
            }
            _ => Trans::None,
        }
    }
}


type DrawPbm = pass::DrawPbm<
    PosNormTangTex,
    AmbientColor,
    MeshComponent,
    MaterialComponent,
    Transform,
    LightComponent,
>;

fn run() -> Result<(), amethyst::Error> {
    let path = format!(
        "{}/examples/06_material/resources/config.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let config = DisplayConfig::load(&path);
    let mut game = Application::build(Example)?
        .with_bundle(
            RenderBundle::new(
                Pipeline::build().with_stage(
                    Stage::with_backbuffer()
                        .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
                        .with_pass(DrawPbm::new()),
                ),
            ).with_config(config),
        )?
        .build()?;
    Ok(game.run())
}

fn main() {
    if let Err(e) = run() {
        println!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn gen_sphere(u: usize, v: usize) -> Vec<PosNormTangTex> {
    SphereUV::new(u, v)
        .vertex(|(x, y, z)| {
            let normal = Vector3::from([x, y, z]).normalize();
            let up = Vector3::from([0.0, 1.0, 0.0]);
            let tangent = normal.cross(up).cross(normal);
            PosNormTangTex {
                a_position: [x, y, z],
                a_normal: normal.into(),
                a_tangent: tangent.into(),
                a_tex_coord: [0.1, 0.1],
            }
        })
        .triangulate()
        .vertices()
        .collect()
}
