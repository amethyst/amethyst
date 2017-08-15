//! Rendering system.

use display_config::DisplayConfig;
use ecs::{Entities, Entity, Fetch, Join, ReadStorage, System, World, WriteStorage};
use ecs::components::*;
use error::{Error, Result};
use renderer::prelude::*;
use super::SystemExt;
use winit::{EventsLoop, get_primary_monitor, WindowBuilder};

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
        Entities<'a>,
        Fetch<'a, Camera>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, LightComponent>,
        WriteStorage<'a, Unfinished<MaterialComponent>>,
        WriteStorage<'a, Unfinished<MeshComponent>>,
        WriteStorage<'a, MaterialComponent>,
        WriteStorage<'a, MeshComponent>,
    );
    fn run(&mut self, (ents, camera, globals, lights, mut umaterials, mut umeshes, mut materials, mut meshes): Self::SystemData) {
        use std::time::Duration;

        /// Finish `Unfinished`
        for (ent, _) in (&*ents, &umaterials.check()).join() {
            println!("Finish material");
            let umaterial = umaterials.remove(ent).expect("Checked");
            let material = umaterial.finish(&mut self.renderer).expect("Why???");
            materials.insert(ent, material);
        }

        for (ent, _) in (&*ents, &umeshes.check()).join() {
            println!("Finish mesh");
            let umesh = umeshes.remove(ent).expect("Checked");
            let mesh = umesh.finish(&mut self.renderer).expect("Why???");
            meshes.insert(ent, mesh);
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
    pub fn new(events: &EventsLoop, config: DisplayConfig, pipe: PipelineBuilder) -> Result<Self>
        where Self: Sized
    {
        let mut window_builder = WindowBuilder::new()
            .with_title(config.title)
            .with_visibility(config.visibility);

        if config.fullscreen {
            window_builder = window_builder.with_fullscreen(get_primary_monitor());
        }
        if let Some((width, height)) = config.dimensions {
            window_builder = window_builder.with_dimensions(width, height)
        }
        if let Some((width, height)) = config.min_dimensions {
            window_builder = window_builder.with_min_dimensions(width, height)
        }
        if let Some((width, height)) = config.max_dimensions {
            window_builder = window_builder.with_max_dimensions(width, height)
        }

        let mut renderer = Renderer::new(events).map_err(|_| Error::System)?;
        let pipe = renderer.create_pipe(pipe).map_err(|_| Error::System)?;

        Ok(RenderSystem {
            pipe: pipe,
            renderer: renderer,
            scene: Scene::default(),
        })
    }
}