//! Rendering system.

use renderer::prelude::*;

use ecs::{Fetch, Join, ReadStorage, System};
use ecs::rendering::components::*;
use ecs::rendering::resources::{Factory, AmbientColor};
use ecs::transform::components::*;

/// Rendering system.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RenderSystem {
    pipe: Pipeline,
    #[derivative(Debug = "ignore")]
    renderer: Renderer,
    scene: Scene,
}

impl RenderSystem {
    /// Create a new render system
    pub fn new(pipe: Pipeline, renderer: Renderer, scene: Scene) -> Self {
        Self {
            pipe,
            renderer,
            scene,
        }
    }
}

impl<'a> System<'a> for RenderSystem {
    type SystemData = (Fetch<'a, Camera>,
     Fetch<'a, Factory>,
     Fetch<'a, AmbientColor>,
     ReadStorage<'a, Transform>,
     ReadStorage<'a, LightComponent>,
     ReadStorage<'a, MaterialComponent>,
     ReadStorage<'a, MeshComponent>);

    fn run(
        &mut self,
        (camera, factory, ambient_color, globals, lights, materials, meshes): Self::SystemData,
    ) {
        use std::time::Duration;

        while let Some(job) = factory.jobs.try_pop() {
            job.exec(&mut self.renderer.factory);
        }

        self.scene.clear();

        for (mesh, material, global) in (&meshes, &materials, &globals).join() {
            self.scene.add_model(Model {
                material: material.0.clone(),
                mesh: mesh.as_ref().clone(),
                pos: global.0.into(),
            });
        }

        self.scene.set_ambient_color(ambient_color.0.clone());

        for light in lights.join() {
            self.scene.add_light(light.0.clone());
        }

        self.scene.add_camera(camera.clone());

        self.renderer.draw(
            &self.scene,
            &self.pipe,
            Duration::from_secs(0),
        );
    }
}