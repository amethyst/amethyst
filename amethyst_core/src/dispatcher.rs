use crate::ecs::{systems::ParallelRunnable, *};
use amethyst_error::Error;

pub use crate::ecs::systems::Builder;

/// A SystemBundle is a structure that adds multiple systems to the [Dispatcher] and loads/unloads all required resources.
pub trait SystemBundle {
    /// [Dispatcher::load] executes this method for all added system bundles.
    fn load(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
        builder: &mut Builder,
    ) -> Result<(), Error>;

    /// [Dispatcher::unload] executes this method for all added system bundles.
    fn unload(&mut self, _world: &mut World, _resources: &mut Resources) -> Result<(), Error> {
        Ok(())
    }
}

/// System bundle that wraps legion's standard system
struct ParallelRunnableBundle<T: ParallelRunnable + 'static> {
    system: Option<T>,
}

impl<T: ParallelRunnable + 'static> SystemBundle for ParallelRunnableBundle<T> {
    fn load(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut Builder,
    ) -> Result<(), Error> {
        builder.with_system(self.system.take().unwrap());
        Ok(())
    }
}

impl<T> From<T> for ParallelRunnableBundle<T>
where
    T: ParallelRunnable + 'static,
{
    fn from(system: T) -> Self {
        Self {
            system: Some(system),
        }
    }
}

/// Builds [Dispatcher] from provided systems and system bundles.
pub struct DispatcherBuilder {
    bundles: Vec<Box<dyn SystemBundle>>,
}

impl Default for DispatcherBuilder {
    fn default() -> Self {
        Self {
            bundles: Vec::with_capacity(20),
        }
    }
}

impl DispatcherBuilder {
    /// Adds [SystemBundle] to the dispatcher. System bundles allow inserting multiple systems
    /// and initialize any required entities or resources.
    pub fn with_bundle<T: SystemBundle + 'static>(&mut self, bundle: T) {
        self.bundles.push(Box::new(bundle));
    }

    /// Adds [SystemBundle] to the dispatcher. System bundles allow inserting multiple systems
    /// and initialize any required entities or resources.
    pub fn add_bundle<T: SystemBundle + 'static>(mut self, bundle: T) -> Self {
        self.with_bundle(bundle);
        self
    }

    /// Adds legion system to the [Dispatcher].
    pub fn with_system<T: ParallelRunnable + 'static>(&mut self, system: T) {
        self.with_bundle(ParallelRunnableBundle::from(system));
    }

    /// Adds legion system to the [Dispatcher].
    pub fn add_system<T: ParallelRunnable + 'static>(mut self, system: T) -> Self {
        self.with_system(system);
        self
    }

    /// Builds [Dispatcher] by calling [SystemBundle::load] on all inserted bundles and constructing a [legion::Schedule].
    pub fn load(
        mut self,
        world: &mut World,
        resources: &mut Resources,
    ) -> Result<Dispatcher, Error> {
        let mut builder = Schedule::builder();

        for bundle in &mut self.bundles {
            bundle.load(world, resources, &mut builder)?;
        }

        Ok(Dispatcher {
            bundles: self.bundles,
            schedule: builder.build(),
        })
    }
}

/// Dispatcher is created by [DispatcherBuilder] and contains [legion::Schedule] used to execute all systems.
pub struct Dispatcher {
    bundles: Vec<Box<dyn SystemBundle>>,
    schedule: Schedule,
}

impl Dispatcher {
    /// Executes systems according to the [legion::Schedule].
    pub fn execute(&mut self, world: &mut World, resources: &mut Resources) {
        self.schedule.execute(world, resources);
    }

    /// Unloads any resources by calling [SystemBundle::unload] for stored system bundles and returns [DispatcherBuilder]
    /// containing the same bundles.
    pub fn unload(
        mut self,
        world: &mut World,
        resources: &mut Resources,
    ) -> Result<DispatcherBuilder, Error> {
        for bundle in &mut self.bundles {
            bundle.unload(world, resources)?;
        }

        Ok(DispatcherBuilder {
            bundles: self.bundles,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    struct MyResource(bool);

    #[test]
    fn dispatcher_loads_and_unloads() {
        struct MyBundle;

        impl SystemBundle for MyBundle {
            fn load(
                &mut self,
                _world: &mut World,
                resources: &mut Resources,
                _builder: &mut Builder,
            ) -> Result<(), Error> {
                resources.insert(MyResource(false));
                Ok(())
            }

            fn unload(
                &mut self,
                _world: &mut World,
                resources: &mut Resources,
            ) -> Result<(), Error> {
                resources.remove::<MyResource>();
                Ok(())
            }
        }

        let mut world = World::default();
        let mut resources = Resources::default();

        // Create dispatcher
        let dispatcher = DispatcherBuilder::default()
            .add_bundle(MyBundle)
            .load(&mut world, &mut resources)
            .unwrap();

        // Ensure that resources were loaded
        assert!(resources.get::<MyResource>().is_some());

        // Unload
        dispatcher.unload(&mut world, &mut resources).unwrap();

        // Ensure that resources were unloaded
        assert!(resources.get::<MyResource>().is_none());
    }

    #[test]
    fn dispatcher_legion_system() {
        let mut world = World::default();
        let mut resources = Resources::default();

        resources.insert(MyResource(false));

        let system = SystemBuilder::new("test")
            .write_resource::<MyResource>()
            .build(|_, _, res, _| {
                res.0 = true;
            });

        let mut dispatcher = DispatcherBuilder::default()
            .add_system(system)
            .load(&mut world, &mut resources)
            .unwrap();

        dispatcher.execute(&mut world, &mut resources);

        assert_eq!(resources.get::<MyResource>().unwrap().0, true);
    }
}
