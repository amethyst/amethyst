
use gfx_hal::Backend;
use gfx_hal::queue::{CommandQueue, Compute, General, Graphics, QueueFamily, QueueGroup, QueueType,
                     RawQueueGroup, Transfer};
use gfx_hal::pool::CommandPool;


struct CommandGroups<B: Backend, C> {
    group: QueueGroup<B, C>,
    pools: Vec<CommandPool<B, C>>,
}

pub struct CommandCenter<B: Backend> {
    transfer: Option<CommandGroups<B, Transfer>>,
    compute: Option<CommandGroups<B, Compute>>,
    graphics: Option<CommandGroups<B, Graphics>>,
    general: Option<CommandGroups<B, General>>,
}

impl<B> CommandCenter<B>
where
    B: Backend,
{
    pub fn new(raw: Vec<RawQueueGroup<B>>) -> Self {
        let mut center = CommandCenter {
            transfer: None,
            compute: None,
            graphics: None,
            general: None,
        };

        for raw in raw {
            match raw.family().queue_type() {
                QueueType::Transfer => {
                    center.transfer = Some(CommandGroups {
                        group: QueueGroup::new(raw),
                        pools: Vec::new(),
                    })
                }

                QueueType::Compute => {
                    center.compute = Some(CommandGroups {
                        group: QueueGroup::new(raw),
                        pools: Vec::new(),
                    })
                }

                QueueType::Graphics => {
                    center.graphics = Some(CommandGroups {
                        group: QueueGroup::new(raw),
                        pools: Vec::new(),
                    })
                }

                QueueType::General => {
                    center.general = Some(CommandGroups {
                        group: QueueGroup::new(raw),
                        pools: Vec::new(),
                    })
                }
            }
        }

        center
    }
}
