extern crate amethyst;

use amethyst::engine::{Application, State, Trans};
use amethyst::processors::rendering::{RenderingProcessor, Renderable, Light, Camera, Projection};
use amethyst::context::Context;
use amethyst::config::Element;
use amethyst::ecs::{World, Join};

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
    fn on_start(&mut self, context: &mut Context, world: &mut World) {
        let (w, h) = context.renderer.get_dimensions().unwrap();

        let eye = [0., 5., 0.];
        let target = [0., 0., 0.];
        let up = [0., 0., 1.];

        let projection = Projection::Perspective {
            fov: 60.0,
            aspect: w as f32 / h as f32,
            near: 1.0,
            far: 100.0,
        };

        let mut camera = Camera::new(projection, eye, target, up);
        camera.activate();

        world.create_now()
            .with(camera)
            .build();

        context.asset_manager.create_constant_texture("dark_blue", [0.0, 0.0, 0.01, 1.]);
        context.asset_manager.create_constant_texture("green", [0.0, 1.0, 0.0, 1.]);
        context.asset_manager.gen_sphere("sphere", 32, 32);

        let sphere = Renderable::new("sphere", "dark_blue", "green");

        world.create_now()
            .with(sphere)
            .build();

        let light = amethyst::renderer::Light {
            color: [1., 1., 1., 1.],
            radius: 1.,
            center: [2., 2., 2.],
            propagation_constant: 0.,
            propagation_linear: 0.,
            propagation_r_square: 1.,
        };

        let light = Light::new(light);

        world.create_now()
            .with(light)
            .build();
    }

    fn update(&mut self, context: &mut Context, world: &mut World) -> Trans {
        use amethyst::context::event::{EngineEvent, Event, VirtualKeyCode};

        let engine_events = context.broadcaster.read::<EngineEvent>();
        for engine_event in engine_events.iter() {
            match engine_event.payload {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return Trans::Quit,
                Event::Closed => return Trans::Quit,
                _ => (),
            }
        }

        let angular_velocity = 2.0; // in radians per second
        self.t += context.delta_time.num_milliseconds() as f32 / 1.0e3;
        let phase = self.t * angular_velocity;

        // Test Transform mutation
        let mut renderables = world.write::<Renderable>();
        for renderable in (&mut renderables).iter() {
            renderable.translation = [phase.sin(), 0.0, phase.cos()];
        }

        let angular_velocity_light = 0.5;
        let phase = self.t * angular_velocity_light;
        // Test Light mutation
        let mut lights = world.write::<Light>();
        for light in (&mut lights).iter() {
            light.light.center = [2.0 * phase.sin(), 2., 2.0 * phase.cos()];
            let angular_velocity_color = 0.7;
            let phase = self.t * angular_velocity_color;
            light.light.color[1] = phase.sin().abs();
        }

        let angular_velocity_camera = 0.3;
        let phase = self.t * angular_velocity_camera;
        // Test Camera mutation
        let mut cameras = world.write::<Camera>();
        for camera in (&mut cameras).iter() {
            camera.eye[1] = 3.0 + 3.0*phase.sin().abs();
        }

        Trans::None
    }
}

fn main() {
    use amethyst::engine::Config;
    let config = Config::from_file(
        format!("{}/config/renderable_example_config.yml",
                env!("CARGO_MANIFEST_DIR"))
        ).unwrap();
    let mut context = Context::new(config.context_config);
    let rendering_processor = RenderingProcessor::new(config.renderer_config, &mut context);
    let mut game = Application::build(Example::new(), context)
                   .with::<RenderingProcessor>(rendering_processor, "rendering_processor", 0)

                   .register::<Renderable>()
                   .register::<Light>()
                   .register::<Camera>()

                   .done();
    game.run();
}
