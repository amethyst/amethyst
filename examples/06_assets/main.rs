extern crate amethyst;
extern crate cgmath;
extern crate obj;
extern crate gfx;

use amethyst::engine::{Application, State, Trans};
use amethyst::context::{ContextConfig, Context};
use amethyst::config::Element;
use amethyst::ecs::{World, Join};
use amethyst::context::asset_manager::{Assets, AssetLoader, AssetLoaderRaw, DirectoryStore, FactoryImpl, Mesh, MeshImpl, Texture};
use amethyst::renderer::VertexPosNormal;
use gfx::traits::FactoryExt;
use cgmath::{InnerSpace, Vector3};
use std::io::BufReader;

struct Obj(obj::Obj);

impl AssetLoaderRaw for Obj {
    fn from_raw(_: &Assets, data: &[u8]) -> Option<Obj> {
        obj::load_obj(BufReader::new(data)).ok().map(|obj| Obj(obj))
    }
}

impl AssetLoader<Mesh> for Obj {
    fn from_data(assets: &mut Assets, obj: Obj) -> Option<Mesh> {
        let factory_impl = assets.get_loader_mut::<FactoryImpl>().expect("Unable to retrieve factory");
        let obj = obj.0;
        let vertices = obj.indices.iter().map(|&index| {
            let vertex = obj.vertices[index as usize];
            let normal = vertex.normal;
            let normal = Vector3::from(normal).normalize();
            VertexPosNormal {
                pos: vertex.position,
                normal: normal.into(),
                tex_coord: [0., 0.],
            }
        }).collect::<Vec<VertexPosNormal>>();

        let mesh_impl = match *factory_impl {
            FactoryImpl::OpenGL { ref mut factory } => {
                let (buffer, slice) = factory.create_vertex_buffer_with_slice(&vertices, ());
                MeshImpl::OpenGL {
                    buffer: buffer,
                    slice: slice,
                }
            }
            #[cfg(windows)]
            FactoryImpl::Direct3D {} => unimplemented!(),
            FactoryImpl::Null => MeshImpl::Null,
        };
        Some(Mesh { mesh_impl: mesh_impl })
    }
}

struct Example;

impl State for Example {
    fn on_start(&mut self, ctx: &mut Context, _: &mut World) {
        use amethyst::renderer::pass::{Clear, DrawShaded};
        use amethyst::renderer::{Layer, Camera, Light};
        use cgmath::Vector3;

        let (w, h) = ctx.renderer.get_dimensions().unwrap();
        let proj = Camera::perspective(60.0, w as f32 / h as f32, 1.0, 100.0);
        let eye = [10.0, 10.0, 0.0];
        let target = [0.0, 3.0, 0.0];
        let up = [0.0, 1.0, 0.0];
        let view = Camera::look_at(eye, target, up);
        let camera = Camera::new(proj, view);

        ctx.renderer.add_scene("main");
        ctx.renderer.add_camera(camera, "main");

        ctx.asset_manager.register_asset::<Mesh>();
        ctx.asset_manager.register_asset::<Texture>();

        ctx.asset_manager.register_loader::<Mesh, Obj>("obj");

        let assets_path = format!("{}/examples/06_assets/resources/assets",
                       env!("CARGO_MANIFEST_DIR"));
        ctx.asset_manager.register_store(DirectoryStore::new(assets_path));

        ctx.asset_manager.create_constant_texture("dark_blue", [0.0, 0.0, 0.2, 1.0]);
        ctx.asset_manager.create_constant_texture("green", [0.0, 1.0, 0.0, 1.0]);
        ctx.asset_manager.load_asset::<Mesh>("Mesh000", "obj");
        ctx.asset_manager.load_asset::<Mesh>("Mesh001", "obj");

        let translation = Vector3::new(0.0, 0.0, 0.0);
        let transform: [[f32; 4]; 4] = cgmath::Matrix4::from_translation(translation).into();
        let fragment = ctx.asset_manager.get_fragment("Mesh000", "dark_blue", "green", transform).unwrap();
        ctx.renderer.add_fragment("main", fragment);

        let fragment = ctx.asset_manager.get_fragment("Mesh001", "dark_blue", "green", transform).unwrap();
        ctx.renderer.add_fragment("main", fragment);

        let light = Light {
            color: [1.0, 1.0, 1.0, 1.0],
            radius: 8.0,
            center: [8.0, 8.0, 8.0],
            propagation_constant: 0.0,
            propagation_linear: 0.0,
            propagation_r_square: 2.0,
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
    let path = format!("{}/examples/06_assets/resources/config.yml",
                       env!("CARGO_MANIFEST_DIR"));
    let config = ContextConfig::from_file(path).unwrap();
    let ctx = Context::new(config);
    let mut game = Application::build(Example, ctx).done();
    game.run();
}
