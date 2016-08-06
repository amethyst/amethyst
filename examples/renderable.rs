extern crate amethyst;
extern crate cgmath;

use cgmath::Vector3;

use amethyst::engine::{Application, State, Trans};
use amethyst::processors::{RenderingProcessor, Renderable, Light, Camera};
use amethyst::processors::rendering::RendererConfig;
use amethyst::context::Context;
use amethyst::config::Element;
use amethyst::ecs::{World, Entity, Join};

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
    fn handle_events(&mut self, events: Vec<Entity>, context: &mut Context, _: &mut World) -> Trans {
        use amethyst::context::event::{EngineEvent, Event, VirtualKeyCode};
        let mut trans = Trans::None;
        let storage = context.broadcaster.read::<EngineEvent>();
        for _event in events {
            let event = storage.get(_event).unwrap();
            let event = &event.payload;
            match *event {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => trans = Trans::Quit,
                Event::Closed => trans = Trans::Quit,
                _ => (),
            }
        }
        trans
    }

    fn on_start(&mut self, context: &mut Context, world: &mut World) {
        let (w, h) = context.renderer.get_dimensions().unwrap();

        let fov = 60.0;
        let aspect = w as f32 / h as f32;
        let near = 1.0;
        let far = 100.0;

        let eye = [0., 5., 0.];
        let target = [0., 0., 0.];
        let up = [0., 0., 1.];

        let mut camera = Camera::new(fov, aspect, near, far,
                                 eye, target, up);
        camera.activate();

        world.create_now()
            .with(camera)
            .build();

        context.asset_manager.create_constant_texture("dark_blue", [0.0, 0.0, 0.01, 1.]);
        context.asset_manager.create_constant_texture("green", [0.0, 1.0, 0.0, 1.]);
        context.asset_manager.gen_sphere("sphere", 32, 32);

        let translation = Vector3::new(0.0, 0.0, 0.0);
        let transform: [[f32; 4]; 4] = cgmath::Matrix4::from_translation(translation).into();

        let sphere = Renderable::new("sphere", "dark_blue", "green", transform);

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
        let angular_velocity = 1.0; // in radians per second
        self.t += context.delta_time.num_milliseconds() as f32 / 1.0e3;
        let phase = self.t * angular_velocity;

        // Test Renderable mutation
        let mut renderables = world.write::<Renderable>();
        for renderable in (&mut renderables).iter() {
            let translation = Vector3::new(phase.sin(), 0.0, phase.cos());
            let transform: [[f32; 4]; 4] = cgmath::Matrix4::from_translation(translation).into();
            renderable.transform = transform;
        }

        // Test Light mutation
        let mut lights = world.write::<Light>();
        for light in (&mut lights).iter() {
            light.light.center = [2.0 * phase.sin(), 2., 2.0 * phase.cos()];
            light.light.color[1] = phase.sin().abs();
        }

        // Test Camera mutation
        let mut cameras = world.write::<Camera>();
        for camera in (&mut cameras).iter() {
            camera.eye[1] = 3.0 + 3.0*phase.sin().abs();
        }

        // Test Light deletion
        if self.t > 5.0 {
            for entity in (&world.entities()).iter() {
                lights.remove(entity.clone());
            }
        }

        // Test Renderable deletion
        if self.t > 10.0 {
            for entity in (&world.entities()).iter() {
                renderables.remove(entity.clone());
            }
        }
        Trans::None
    }
}

fn main() {
    use amethyst::context::Config;
    let config = Config::from_file("../config/window_example_config.yml").unwrap();
    let renderer_config = RendererConfig::from_file("../config/renderer_config.yml").unwrap();
    let mut context = Context::new(config);
    let rendering_processor = RenderingProcessor::new(renderer_config, &mut context);
    let mut game = Application::build(Example::new(), context)
                   .with::<RenderingProcessor>(rendering_processor, "rendering_processor", 0)
                   .done();
    game.run();
}
