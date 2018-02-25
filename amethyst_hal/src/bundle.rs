use core::{ECSBundle, Error as BundleError};
use hal::Instance;
use hal::adapter::PhysicalDevice;
use hal::queue::{General, QueueFamily, QueueType};
use mem::SmartAllocator;
use shred::DispatcherBuilder;
use specs::World;
use std::marker::PhantomData;
use std::string::ToString;

use factory::{BackendEx, Factory};
use system::RenderSystem;

/// Render bundle initialize rendering systems
/// puts `Factory` to the `World`
/// and adds `RenderSystem` to the `DispatcherBuilder`
pub struct RenderBundle<B> {
    queues: usize,
    pd: PhantomData<fn() -> B>,
}

/// A bundle of ECS components, resources and systems.
impl<'a, 'b, B> ECSBundle<'a, 'b> for RenderBundle<B>
where
    B: BackendEx,
{
    /// Build and add ECS resources, register components, add systems etc to the Application.
    fn build(
        self,
        world: &mut World,
        dispatcher: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>, BundleError> {
        #[cfg(feature = "metal")]
        let autorelease_pool = {
            if is::<B, metal::Backend>() {
                Some(unsafe { metal::AutoreleasePool::new() });
            } else {
                None
            }
        };

        let instance = B::init();
        let mut adapter = instance.enumerate_adapters().remove(0);
        info!("Get adapter {:#?}", adapter.info);

        info!(
            "Device features: {:#?}",
            adapter.physical_device.features()
        );
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

        let system = RenderSystem::new(queue_group);
        let factory = Factory::new(instance, adapter.physical_device, device, allocator);

        world.add_resource(factory);

        Ok(dispatcher.add_thread_local(system))
    }
}
