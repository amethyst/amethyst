use amethyst::{
    animation::{
        get_animation_set, AnimationBundle, AnimationCommand, AnimationHierarchy, AnimationSet,
        EndControl,
    },
    assets::{prefab::Prefab, DefaultLoader, Handle, Loader, LoaderBundle, ProgressCounter},
    core::{ecs::CommandBuffer, transform::TransformBundle, Transform},
    ecs::{DispatcherBuilder, Entity, IntoQuery},
    gltf::bundle::GltfBundle,
    renderer::{types::DefaultBackend, RenderPbr3D, RenderSkybox, RenderToWindow, RenderingBundle},
    utils::application_root_dir,
    Application, GameData, SimpleState, SimpleTrans, StateData, Trans,
};

struct GltfExample {
    pub progress_counter: Option<ProgressCounter>,
}

impl Default for GltfExample {
    fn default() -> Self {
        GltfExample {
            progress_counter: Some(ProgressCounter::default()),
        }
    }
}

impl SimpleState for GltfExample {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData {
            world, resources, ..
        } = data;
        let loader = resources.get::<DefaultLoader>().unwrap();
        let t: Handle<Prefab> = loader.load("gltf/supercube.glb");
        world.push((t,));
    }

    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        let StateData { world, .. } = data;

        let mut query = <(
            Entity,
            &AnimationSet<usize, Transform>,
            &AnimationHierarchy<Transform>,
        )>::query();
        let mut buffer = CommandBuffer::new(world);

        if let Some(ref progress_counter) = self.progress_counter {
            // Checks progress
            if progress_counter.is_complete() {
                let (query_world, mut subworld) = world.split_for_query(&query);
                for (entity, animation_set, _t) in query.iter(&query_world) {
                    // Creates a new AnimationControlSet for the entity
                    if let Some(control_set) =
                        get_animation_set(&mut subworld, &mut buffer, *entity)
                    {
                        if control_set.is_empty() {
                            control_set.add_animation(
                                0,
                                &animation_set.get(&0).unwrap(),
                                EndControl::Loop(None),
                                1.0,
                                AnimationCommand::Start,
                            );
                            self.progress_counter = None;
                        }
                    }
                }
            }
        }
        buffer.flush(world);

        Trans::None
    }
}

fn main() -> Result<(), amethyst::Error> {
    let config = Default::default();

    amethyst::start_logger(config);
    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("config/display.ron");
    let assets_dir = app_root.join("assets/");

    let mut dispatcher = DispatcherBuilder::default();
    dispatcher
        .add_bundle(LoaderBundle)
        .add_bundle(GltfBundle)
        .add_bundle(TransformBundle)
        .add_bundle(AnimationBundle::<i32, Transform>::default()) // This is the animations coming from GLTF
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderToWindow::from_config_path(display_config_path)?)
                .with_plugin(RenderPbr3D::default())
                .with_plugin(RenderSkybox::default()),
        );

    let game = Application::new(assets_dir, GltfExample::default(), dispatcher)?;
    game.run();
    Ok(())
}
