mod family;

use std::cmp::min;
use std::collections::VecDeque;

use gfx_hal::{Backend, Device};
use gfx_hal::device::WaitFor;
use gfx_hal::pool::CommandPool;
use gfx_hal::queue::{CommandQueue, Compute, General, Graphics, QueueFamily, QueueGroup, QueueType,
                     RawQueueGroup, Supports, Transfer};

use smallvec::SmallVec;

use self::family::Family;
use epoch::{CurrentEpoch, Epoch};


pub trait Execution<B: Backend, C> {
    fn execute(
        self,
        queue: &mut CommandQueue<B, C>,
        pools: &mut [CommandPool<B, C>],
        fence: &B::Fence,
        device: &B::Device,
    );
}


pub struct CommandCenter<B: Backend> {
    transfer: Option<Family<B, Transfer>>,
    compute: Option<Family<B, Compute>>,
    graphics: Option<Family<B, Graphics>>,
    general: Option<Family<B, General>>,
    fences: Vec<B::Fence>,
}

impl<B> CommandCenter<B>
where
    B: Backend,
{
    /// Create new `CommandCenter` from `RawQueueGroup`s
    pub fn new(raw: Vec<RawQueueGroup<B>>) -> Self {
        let mut center = CommandCenter {
            transfer: None,
            compute: None,
            graphics: None,
            general: None,
            fences: Vec::new(),
        };

        for raw in raw {
            match raw.family().queue_type() {
                QueueType::Transfer => center.transfer = Some(Family::new(QueueGroup::new(raw))),
                QueueType::Compute => center.compute = Some(Family::new(QueueGroup::new(raw))),
                QueueType::Graphics => center.graphics = Some(Family::new(QueueGroup::new(raw))),
                QueueType::General => center.general = Some(Family::new(QueueGroup::new(raw))),
            }
        }
        center
    }

    /// Execute graphics operation
    ///
    /// This operation can't start before `start` epoch.
    /// This operation will be finished for `span` epochs.
    /// Technically epochs won't advance to `start + span` before this operation get finished.
    /// This means that it will wait until all other operations with `other_start + other_span < this_start` are finished.
    ///
    /// Typically `span` of the graphics operations equals to number of surface framebuffers.
    /// And each new instance of operation have `start` equals to prev's `start` + 1.
    /// So that N instances of same graphics operation can run simulteneously and (N+1)th will wait for 1st to finish.
    ///
    /// `span` can't be less than 1
    pub fn execute_graphics<E>(
        &mut self,
        execution: E,
        start: Epoch,
        span: usize,
        current: &mut CurrentEpoch,
        device: &B::Device,
    ) where
        E: Execution<B, Graphics> + Execution<B, General>,
    {
        profile_scope!("CommandCenter::execute_graphics");
        self.wait(device, current, start);
        let index = span - 1;

        let fence = self.fences
            .pop()
            .unwrap_or_else(|| device.create_fence(false));

        if let Some(ref mut family) = self.graphics.as_mut() {
            profile_scope!("CommandCenter::execute_graphics :: execute");
            let (mut queue, mut pools) = family.acquire(1, device);
            execution.execute(&mut queue, &mut pools, &fence, device);
            family.release(queue, pools, fence, index, device);
            return;
        }

        if let Some(ref mut family) = self.general.as_mut() {
            profile_scope!("CommandCenter::execute_graphics :: execute");
            let (mut queue, mut pools) = family.acquire(1, device);
            execution.execute(&mut queue, &mut pools, &fence, device);
            family.release(queue, pools, fence, index, device);
            return;
        }
    }

    /// Check finished operation and advance current epoch.
    fn cleanup(&mut self, device: &B::Device, current: &mut CurrentEpoch) {
        profile_scope!("CommandCenter::cleanup");
        let mut ready = 32;

        if let Some(ref mut family) = self.transfer.as_mut() {
            match family.check_ready(device, &mut self.fences) {
                Some(r) => ready = min(ready, r),
                None => {}
            }
        }
        if let Some(ref mut family) = self.compute.as_mut() {
            match family.check_ready(device, &mut self.fences) {
                Some(r) => ready = min(ready, r),
                None => {}
            }
        }
        if let Some(ref mut family) = self.graphics.as_mut() {
            match family.check_ready(device, &mut self.fences) {
                Some(r) => ready = min(ready, r),
                None => {}
            }
        }
        if let Some(ref mut family) = self.general.as_mut() {
            match family.check_ready(device, &mut self.fences) {
                Some(r) => ready = min(ready, r),
                None => {}
            }
        }

        if let Some(ref mut family) = self.transfer.as_mut() {
            family.shift_ready(ready);
        }
        if let Some(ref mut family) = self.compute.as_mut() {
            family.shift_ready(ready);
        }
        if let Some(ref mut family) = self.graphics.as_mut() {
            family.shift_ready(ready);
        }
        if let Some(ref mut family) = self.general.as_mut() {
            family.shift_ready(ready);
        }

        current.advance(ready as u64);
    }

    fn wait_step(&mut self, device: &B::Device, depth: usize) {
        profile_scope!("CommandCenter::wait_step");
        let mut fences: SmallVec<[_; 32]> = SmallVec::new();
        self.transfer
            .as_mut()
            .map(|family| family.collect_fences(&mut fences, depth));
        self.compute
            .as_mut()
            .map(|family| family.collect_fences(&mut fences, depth));
        self.graphics
            .as_mut()
            .map(|family| family.collect_fences(&mut fences, depth));
        self.general
            .as_mut()
            .map(|family| family.collect_fences(&mut fences, depth));

        if !device.wait_for_fences(&fences[..], WaitFor::All, 10000) {
            panic!("Expect to finish operations for 10 secs");
        }
    }

    fn wait(&mut self, device: &B::Device, current: &mut CurrentEpoch, epoch: Epoch) {
        profile_scope!("CommandCenter::wait");
        self.cleanup(device, current);

        while epoch > current.now() {
            self.wait_step(device, (epoch.0 - current.now().0) as usize);
            self.cleanup(device, current);
        }
    }
}
