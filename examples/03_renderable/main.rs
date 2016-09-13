extern crate amethyst;

use amethyst::engine::{Application, Planner, State, Trans};
use amethyst::processors::rendering::{RenderingProcessor, Renderable, Light, Camera, Projection};
use amethyst::context::Context;
use amethyst::config::Element;
use amethyst::ecs::Join;

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
    fn on_start(&mut self, context: &mut Context, planner: &mut Planner) {
        let (w, h) = context.renderer.get_dimensions().unwrap();
        let world = planner.mut_world();

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

    fn update(&mut self, context: &mut Context, planner: &mut Planner) -> Trans {
        use amethyst::context::event::{EngineEvent, Event, VirtualKeyCode};

        let engine_events = context.broadcaster.read::<EngineEvent>();
        for engine_event in engine_events.iter() {
            match engine_event.payload {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return Trans::Quit,
                Event::Closed => return Trans::Quit,
                _ => (),
            }
        }

        self.t += context.delta_time.subsec_nanos() as f32 / 1.0e9;
        let time = self.t;

        // Test Transform mutation
        planner.run_custom(move |arg| {
            let mut renderables = arg.fetch(|w| w.write::<Renderable>());
            let angular_velocity = 2.0; // in radians per second
            let phase = time * angular_velocity;
            let offset = [phase.sin(), 0.0, phase.cos()];
            for renderable in (&mut renderables).iter() {
                renderable.translation = offset;
            }
        });

        // Test Light mutation
        planner.run_custom(move |arg| {
            let mut lights = arg.fetch(|w| w.write::<Light>());
            let angular_velocity = 0.5; // in radians per second
            let phase = time * angular_velocity;
            let center = [2.0 * phase.sin(), 2., 2.0 * phase.cos()];
            let angular_velocity_color = 0.7;
            let phase_color = time * angular_velocity_color;
            for light in (&mut lights).iter() {
                light.light.center = center;
                light.light.color[1] = phase_color.sin().abs();
            }
        });

        // Test Camera mutation
        planner.run_custom(move |arg| {
            let mut cameras = arg.fetch(|w| w.write::<Camera>());
            let angular_velocity = 0.3; // in radians per second
            let phase = time * angular_velocity;
            for camera in (&mut cameras).iter() {
                camera.eye[1] = 3.0 + 3.0*phase.sin().abs();
            }
        });

        Trans::None
    }
}

fn main() {
    use amethyst::engine::Config;
    let path = format!("{}/examples/03_renderable/resources/config.yml",
                    env!("CARGO_MANIFEST_DIR"));
    let config = Config::from_file(path).unwrap();
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
