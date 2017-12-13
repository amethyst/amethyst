use std::cmp::min;
use std::collections::VecDeque;

use gfx_hal::{Backend, Device};
use gfx_hal::pool::{CommandPool, CommandPoolCreateFlags};
use gfx_hal::queue::{CommandQueue, QueueGroup};

/// Family of the queues
/// It holds whole `QueueGroup` with `CommandQueue`s
/// and all allocated `CommandPool`s associated with this groups.
pub struct Family<B: Backend, C> {
    group: QueueGroup<B, C>,
    pools: Vec<CommandPool<B, C>>,
    executions: VecDeque<Vec<(Vec<CommandPool<B, C>>, B::Fence)>>,
    clean_vecs: Vec<Vec<(Vec<CommandPool<B, C>>, B::Fence)>>,
}

impl<B, C> Family<B, C>
where
    B: Backend,
{
    ///  Create new `Family` from `QueueGroup`
    pub fn new(group: QueueGroup<B, C>) -> Self {
        Family {
            group,
            pools: Vec::new(),
            executions: VecDeque::new(),
            clean_vecs: Vec::new(),
        }
    }

    /// Acquire queue and specified numbers of unused pools.
    pub fn acquire(
        &mut self,
        pools: usize,
        device: &B::Device,
    ) -> (CommandQueue<B, C>, Vec<CommandPool<B, C>>) {
        let ref mut group = self.group;
        let queue = group.queues.pop().unwrap();
        let len = self.pools.len();
        self.pools.extend((len..pools).map(|_| {
            device.create_command_pool_typed(group, CommandPoolCreateFlags::empty(), 1)
        }));
        let len = self.pools.len();
        let pools = self.pools.drain((len - pools)..).collect();

        (queue, pools)
    }

    /// Release queue and pools.
    /// `Family` will use specified `B::Fence` to check when pools can be reused.
    /// fence and pools will be stored at specified offset.
    /// Later `check_ready` will return an offset until which all fences are signaled.
    pub fn release(
        &mut self,
        queue: CommandQueue<B, C>,
        pools: Vec<CommandPool<B, C>>,
        fence: B::Fence,
        offset: usize,
        device: &B::Device,
    ) {
        self.group.queues.push(queue);
        if !device.get_fence_status(&fence) {
            let ref mut clean_vecs = self.clean_vecs;
            let len = self.executions.len();
            self.executions
                .extend((len..offset + 1).map(|_| clean_vecs.pop().unwrap_or_else(|| Vec::new())));
            self.executions[offset].push((pools, fence));
        }
    }

    /// Check until which offset fences are signaled.
    /// All signaled fences are removed and put to the `fences`
    /// Returns `None` if there is no pools in use at all.
    pub fn check_ready(&mut self, device: &B::Device, fences: &mut Vec<B::Fence>) -> Option<usize> {
        // TODO: Use `drain_filter` when stabilized.

        if self.executions.is_empty() {
            return None;
        }

        let mut count = 0;
        for execs in self.executions.iter_mut() {
            let mut removed = 0;
            for i in 0..execs.len() {
                let i = i - removed;
                if device.get_fence_status(&execs[i].1) {
                    let (p, f) = execs.swap_remove(i);
                    removed += 1;
                    self.pools.extend(p.into_iter().map(|mut p| {
                        p.reset();
                        p
                    }));
                    fences.push(f);
                }
            }
            if !execs.is_empty() {
                break;
            }
            count += 1;
        }

        Some(count)
    }

    /// Shift inner pools'n'fences queue.
    /// `ready` should be not greater then value returned by `check_ready`
    /// If `check_ready` returned `None` - `ready` can be anything.
    pub fn shift_ready(&mut self, ready: usize) {
        let len = self.executions.len();
        for execs in self.executions.drain(..min(ready, len)) {
            assert!(execs.is_empty());
            self.clean_vecs.push(execs);
        }
    }

    /// Collect fences in the front of queue.
    ///
    ///
    pub fn collect_fences<'a, E>(&'a self, fences: &mut E, depth: usize)
    where
        E: Extend<&'a B::Fence>,
    {
        fences.extend(
            self.executions
                .iter()
                .take(depth)
                .flat_map(|x| x.iter())
                .map(|x| &x.1),
        )
    }
}
