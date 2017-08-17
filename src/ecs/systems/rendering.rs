//! Rendering system.

use ecs::{Fetch, Join, ReadStorage, System};
use ecs::components::*;
use ecs::resources::Factory as FactoryRes;
use error::{Error, Result};
use renderer::prelude::*;
use winit::EventsLoop;

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
        Fetch<'a, FactoryRes>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, LightComponent>,
        ReadStorage<'a, MaterialComponent>,
        ReadStorage<'a, MeshComponent>,
    );

    fn run(&mut self, (camera, factory_res, globals, lights, materials, meshes): Self::SystemData) {
        use std::time::Duration;

        while let Some(job) = factory_res.jobs.try_pop() {
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

impl RenderSystem {
    /// Create new `RenderSystem`
    /// It creates window and do render into it
    pub fn new(events: &EventsLoop, pipe: PipelineBuilder) -> Result<Self>
        where Self: Sized
    {
        let mut renderer = Renderer::new(events).map_err(|_| Error::System)?;
        let pipe = renderer.create_pipe(pipe).map_err(|_| Error::System)?;

        Ok(RenderSystem {
            pipe: pipe,
            renderer: renderer,
            scene: Scene::default(),
        })
    }
}
