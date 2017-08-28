//! Rendering system.

use winit::EventsLoop;

use assets::BoxedErr;
use ecs::{Fetch, Join, ReadStorage, System, World};
use ecs::transform::components::*;
use ecs::rendering::components::*;
use ecs::rendering::resources::{Factory, AmbientColor};

use error::{Error, Result};
use renderer::prelude::*;
use renderer::Config as DisplayConfig;
use renderer::Rgba;

use ecs::SystemExt;

/// Rendering system.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RenderSystem {
    pipe: Pipeline,
    #[derivative(Debug = "ignore")]
    renderer: Renderer,
    scene: Scene,
}

impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        Fetch<'a, Camera>,
        Fetch<'a, Factory>,
        Fetch<'a, AmbientColor>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, LightComponent>,
        ReadStorage<'a, MaterialComponent>,
        ReadStorage<'a, MeshComponent>,
    );

    fn run(&mut self, (camera, factory, ambient_color, globals, lights, materials, meshes): Self::SystemData) {
        use std::time::Duration;

        while let Some(job) = factory.jobs.try_pop() {
            job.exec(&mut self.renderer.factory);
        }

        self.scene.clear();

        for (mesh, material, global) in (&meshes, &materials, &globals).join() {
            self.scene.add_model(Model {
                material: material.0.clone(),
                mesh: mesh.as_ref().clone(),
                pos: global.0.into()
            });
        }

        self.scene.set_ambient_color(ambient_color.0.clone());

        for light in lights.join() {
            self.scene.add_light(light.0.clone());
        }

        self.scene.add_camera(camera.clone());

        self.renderer.draw(&self.scene, &self.pipe, Duration::from_secs(0));
    }
}

impl<'a, 'b> SystemExt<'a, (&'b EventsLoop, PipelineBuilder, Option<DisplayConfig>)> for RenderSystem {
    /// Create new `RenderSystem`
    /// It creates window and do render into it
    fn build((events, pipe, config): (&'b EventsLoop, PipelineBuilder, Option<DisplayConfig>), world: &mut World) -> Result<Self> {
        let mut renderer = Renderer::build(events);
        if let Some(config) = config {
            renderer.with_config(config);
        }
        let mut renderer =  renderer.build().map_err(|err| Error::System(BoxedErr::new(err)))?;
        let pipe = renderer.create_pipe(pipe).map_err(|err| Error::System(BoxedErr::new(err)))?;

        use cgmath::Deg;
        use renderer::{Camera, Projection};

        let cam = Camera {
            eye: [0.0, 0.0, -4.0].into(),
            proj: Projection::perspective(1.3, Deg(60.0)).into(),
            forward: [0.0, 0.0, 1.0].into(),
            right: [1.0, 0.0, 0.0].into(),
            up: [0.0, 1.0, 0.0].into(),
        };

        world.add_resource(Factory::new());
        world.add_resource(cam);
        world.add_resource(AmbientColor(Rgba::from([0.01; 3])));
        world.register::<Transform>();
        world.register::<LightComponent>();
        world.register::<MaterialComponent>();
        world.register::<MeshComponent>();

        Ok(RenderSystem {
            pipe: pipe,
            renderer: renderer,
            scene: Scene::default(),
        })
    }
}
