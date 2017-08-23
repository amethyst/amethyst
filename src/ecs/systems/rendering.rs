//! Rendering system.

use std::sync::Arc;


use winit::EventsLoop;


use ecs::{Fetch, Join, ReadStorage, System, World};
use ecs::components::*;
use ecs::resources::Factory;
use error::{Error, Result};
use renderer::prelude::*;
use super::SystemExt;

/// Rendering system.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RenderSystem {
    pipe: Pipeline,
    #[derivative(Debug = "ignore")]
    renderer: Renderer,
    scene: Scene,
    #[derivative(Debug = "ignore")]
    factory: Arc<Factory>,
}

impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        Fetch<'a, Camera>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, LightComponent>,
        ReadStorage<'a, MaterialComponent>,
        ReadStorage<'a, MeshComponent>,
    );

    fn run(&mut self, (camera, globals, lights, materials, meshes): Self::SystemData) {
        use std::time::Duration;

        while let Some(job) = self.factory.jobs.try_pop() {
            job.exec(&mut self.renderer.factory);
        }

        self.scene.clear();

        for (mesh, material, global) in (&meshes, &materials, &globals).join() {
            self.scene.add_model(Model {
                material: material.0.clone(),
                mesh: mesh.0.clone(),
                pos: global.0.into()
            });
        }

        for light in lights.join() {
            self.scene.add_light(light.0.clone());
        }

        self.scene.add_camera(camera.clone());

        self.renderer.draw(&self.scene, &self.pipe, Duration::from_secs(0));
    }
}

impl<'a, 'b> SystemExt<'a, (&'b EventsLoop, PipelineBuilder)> for RenderSystem {
    /// Create new `RenderSystem`
    /// It creates window and do render into it
    fn build((events, pipe): (&'b EventsLoop, PipelineBuilder), world: &mut World) -> Result<Self> {
        let mut renderer = Renderer::new(events).map_err(|_| Error::System)?;
        let pipe = renderer.create_pipe(pipe).map_err(|_| Error::System)?;

        use cgmath::Deg;
        use renderer::{Camera, Projection};
        use ecs::components::{LightComponent, MaterialComponent, MeshComponent, TextureComponent,
                              TextureContext, Transform};
        use ecs::resources::Factory;
        use assets::Loader;

        let cam = Camera {
            eye: [0.0, 0.0, -4.0].into(),
            proj: Projection::perspective(1.3, Deg(60.0)).into(),
            forward: [0.0, 0.0, 1.0].into(),
            right: [1.0, 0.0, 0.0].into(),
            up: [0.0, 1.0, 0.0].into(),
        };

        world.add_resource(cam);
        world.register::<LightComponent>();
        world.register::<MaterialComponent>();
        world.register::<MeshComponent>();
        world.register::<TextureComponent>();
        world.register::<Transform>();

        let factory = Arc::new(Factory::new());

        // No way to know if `Loaded` was added. Just hope it was
        let mut loader = world.write_resource::<Loader>();
        loader.register(TextureContext::new(factory.clone()));
        // TODO: Other contexts

        Ok(RenderSystem {
            pipe: pipe,
            renderer: renderer,
            scene: Scene::default(),
            factory: factory,
        })
    }
}
