extern crate amethyst;
extern crate genmesh;
extern crate cgmath;

use amethyst::engine::{Application, State, Trans};
use amethyst::config::Element;
use amethyst::ecs::{World, Join};
use amethyst::gfx_device::DisplayConfig;
use amethyst::asset_manager::AssetManager;
use amethyst::event::WindowEvent;
use amethyst::renderer::{VertexPosNormal, Pipeline};

use self::genmesh::generators::{SphereUV};
use self::genmesh::{MapToVertices, Triangulate, Vertices};
use self::cgmath::{Vector3, InnerSpace};

struct Example {
    t: f32,
}

impl Example {
    pub fn new() -> Example {
        Example {
            t: 0.0,
        }
    }
}

impl State for Example {
    fn on_start(&mut self, world: &mut World, asset_manager: &mut AssetManager, pipeline: &mut Pipeline) {
        use amethyst::renderer::pass::{Clear, DrawShaded};
        use amethyst::renderer::{Layer, PointLight};
        use amethyst::world_resources::camera::{Projection, Camera};
        use amethyst::world_resources::ScreenDimensions;
        use amethyst::components::transform::{LocalTransform, Transform};
        use amethyst::components::rendering::{Texture, Mesh, Renderable};

        let layer =
            Layer::new("main",
                        vec![
                            Clear::new([0.0, 0.0, 0.0, 1.0]),
                            DrawShaded::new("main", "main"),
                        ]);
        pipeline.layers = vec![layer];
        {
            let dimensions = world.read_resource::<ScreenDimensions>();
            let mut camera = world.write_resource::<Camera>();
            camera.projection = Projection::Perspective {
                fov: 90.0,
                aspect_ratio: dimensions.aspect_ratio,
                near: 0.1,
                far: 100.0,
            };
            camera.eye = [5.0, 0.0, 0.0];
            camera.target = [0.0, 0.0, 0.0];
        }
        let sphere_vertices = gen_sphere(32, 32);
        asset_manager.register_asset::<Mesh>();
        asset_manager.register_asset::<Texture>();
        asset_manager.load_asset_from_data::<Mesh, Vec<VertexPosNormal>>("sphere", sphere_vertices);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("blue", [0.0, 0.0, 1.0, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("green", [0.0, 1.0, 0.0, 1.0]);
        let sphere = asset_manager.create_renderable("sphere", "blue", "green", "green", 1.0).unwrap();
        world.create_now()
            .with::<Renderable>(sphere)
            .with::<LocalTransform>(LocalTransform::default())
            .with::<Transform>(Transform::default())
            .build();
        let light = PointLight {
            center: [2.0, 2.0, 2.0],
            ..Default::default()
        };
        world.create_now()
            .with::<PointLight>(light)
            .build();
    }

    fn update(&mut self, world: &mut World, _: &mut AssetManager, _: &mut Pipeline) -> Trans {
        use amethyst::renderer::PointLight;
        use amethyst::world_resources::Camera;
        use amethyst::world_resources::Time;
        use amethyst::components::transform::LocalTransform;

        let time = world.read_resource::<Time>();
        let angular_velocity = 2.0; // in radians per second
        self.t += time.delta_time.subsec_nanos() as f32 / 1.0e9;
        let phase = self.t * angular_velocity;

        // Test Transform mutation
        let mut locals = world.write::<LocalTransform>();
        for local in (&mut locals).iter() {
            local.translation = [phase.sin(), 0.0, phase.cos()];
        }

        let angular_velocity_light = 0.5;
        let phase = self.t * angular_velocity_light;
        // Test Light mutation
        let mut lights = world.write::<PointLight>();
        for light in (&mut lights).iter() {
            light.center = [2.0 * phase.sin(), 2., 2.0 * phase.cos()];
            let angular_velocity_color = 0.7;
            let phase = self.t * angular_velocity_color;
            light.color[1] = phase.sin().abs();
        }

        let angular_velocity_camera = 0.3;
        let phase = self.t * angular_velocity_camera;
        // Test Camera mutation
        let mut camera = world.write_resource::<Camera>();
        camera.eye[1] = 3.0 + 3.0*phase.sin().abs();

        Trans::None
    }

    fn handle_events(&mut self, events: &[WindowEvent], _: &mut World, _: &mut AssetManager, _: &mut Pipeline) -> Trans {
        use amethyst::event::*;
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
    let path = format!("{}/examples/01_window/resources/config.yml",
                       env!("CARGO_MANIFEST_DIR"));
    let display_config = DisplayConfig::from_file(path).unwrap();
    let mut game = Application::build(Example::new(), display_config).done();
    game.run();
}

fn gen_sphere(u: usize, v: usize) -> Vec<VertexPosNormal> {
    let data: Vec<VertexPosNormal> = SphereUV::new(u, v)
        .vertex(|(x, y, z)| {
            VertexPosNormal {
                pos: [x, y, z],
                normal: Vector3::new(x, y, z).normalize().into(),
                tex_coord: [0., 0.],
            }
        })
        .triangulate()
        .vertices()
        .collect();
    data
}
