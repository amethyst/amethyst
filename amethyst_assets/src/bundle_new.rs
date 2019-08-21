use crate::experimental::{Loader, DefaultLoader};
use amethyst_error::Error;
use amethyst_core::{
    ecs::prelude::{RunNow, DispatcherBuilder, World, WorldExt},
    SystemBundle, 
};

struct LoaderSystem;
impl<'a> RunNow<'a> for LoaderSystem {
    fn setup(&mut self, _world: &mut World) {
        // LoaderSystem is set up in LoaderBundle since it needs access to DispatcherBuilder too.
    }
    fn run_now(&mut self, world: &'a World) {
        let mut loader = world.write_resource::<DefaultLoader>();
        loader.process(world).expect("Error in Loader processing"); 
    }
}

/// Bundle that initializes Loader as well as related processing systems and resources
pub struct LoaderBundle;

impl SystemBundle<'static, 'static> for LoaderBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'static, 'static>,
    ) -> Result<(), Error> {
        builder.add_thread_local(LoaderSystem);
        let mut loader = DefaultLoader::default();
        loader.init_world(world);
        loader.init_dispatcher(builder);
        world.insert(loader);
        Ok(())
    }
}