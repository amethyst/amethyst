use core::{ECSBundle, Error as BundleError};
use hal::Instance;
use hal::adapter::PhysicalDevice;
use hal::queue::{General, QueueFamily, QueueType};
use mem::SmartAllocator;
use shred::{DispatcherBuilder, Resources};
use specs::World;
use xfg::Pass;

use std::marker::PhantomData;
use std::string::ToString;

use backend::BackendEx;
use factory::Factory;
use renderer::Renderer;
use system::RenderSystem;

const STAGING_TRESHOLD: usize = 32 * 1024; // 32kb

/// Render bundle initialize rendering systems
/// puts `Factory` to the `World`
/// and adds `RenderSystem` to the `DispatcherBuilder`
pub struct RenderBundle<B, P> {
    queues: usize,
    pd: PhantomData<fn() -> (B, P)>,
}

/// A bundle of ECS components, resources and systems.
impl<'a, 'b, B, P> ECSBundle<'a, 'b> for RenderBundle<B, P>
where
    B: BackendEx,
    P: Send + Sync + for<'c> Pass<B, &'c Resources> + 'static,
{
    /// Build and add ECS resources, register components, add systems etc to the Application.
    fn build(
        self,
        world: &mut World,
        dispatcher: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>, BundleError> {
        let instance = B::init();
        let mut adapter = instance.enumerate_adapters().remove(0);
        info!("Get adapter {:#?}", adapter.info);

        info!("Device features: {:#?}", adapter.physical_device.features());
        info!("Device limits: {:#?}", adapter.physical_device.limits());

        let (device, queue_group) = {
            info!("Queue families: {:#?}", adapter.queue_families);
            let qf = adapter
                .queue_families
                .drain(..)
                .filter(|family| family.queue_type() == QueueType::General)
                .next()
                .ok_or(format!("Can't find General queue family"))?;
            let mut gpu = adapter
                .physical_device
                .open(vec![(&qf, vec![1.0; self.queues])])
                .map_err(|err| err.to_string())?;
            let queue_group = gpu.queues
                .take::<General>(qf.id())
                .expect("This group was requested");
            (gpu.device, queue_group)
        };
        info!("Logical device created");

        let allocator = SmartAllocator::<B>::new(
            adapter.physical_device.memory_properties(),
            32,
            32,
            32,
            1024 * 1024 * 64,
        );
        info!("Allocator created: {:#?}", allocator);

        let factory = Factory::new(
            instance,
            adapter.physical_device,
            device,
            allocator,
            STAGING_TRESHOLD,
            queue_group.family(),
        );
        let renderer = Renderer::<B, P>::new(queue_group);

        world.add_resource(factory);
        world.add_resource(renderer);

        Ok(dispatcher.add_thread_local(RenderSystem::<B, P>::new()))
    }
}
