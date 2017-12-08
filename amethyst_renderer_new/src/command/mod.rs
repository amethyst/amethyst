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

pub trait GeneralExecution<B: Backend>: Execution<B, General> {}
impl<T, B> GeneralExecution<B> for T
where
    B: Backend,
    T: Execution<B, General>,
{
}

pub trait GraphicsExecution<B: Backend>
    : Execution<B, Graphics> + GeneralExecution<B> {
}
impl<T, B> GraphicsExecution<B> for T
where
    B: Backend,
    T: Execution<B, Graphics> + Execution<B, General>,
{
}

pub trait TransferExecution<B: Backend>
    : Execution<B, Transfer> + GraphicsExecution<B> {
}
impl<T, B> TransferExecution<B> for T
where
    B: Backend,
    T: Execution<B, Transfer> + Execution<B, Graphics> + Execution<B, General>,
{
}


pub trait Execution<B: Backend, C> {
    fn execute(
        self,
        queue: &mut CommandQueue<B, C>,
        pools: &mut [CommandPool<B, C>],
        current: &CurrentEpoch,
        fence: &B::Fence,
        device: &B::Device,
    ) -> Epoch;
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
        current: &mut CurrentEpoch,
        device: &B::Device,
    ) where
        E: GraphicsExecution<B>,
    {
        profile_scope!("CommandCenter::execute_graphics");
        self.wait(device, current, start);

        let fence = self.fences
            .pop()
            .unwrap_or_else(|| device.create_fence(false));

        if let Some(ref mut family) = self.graphics.as_mut() {
            profile_scope!("CommandCenter::execute_graphics :: execute");
            let (mut queue, mut pools) = family.acquire(1, device);
            let finish = execution.execute(&mut queue, &mut pools, &*current, &fence, device);
            family.release(
                queue,
                pools,
                fence,
                (finish.0 - current.now().0) as usize,
                device,
            );
            return;
        }

        if let Some(ref mut family) = self.general.as_mut() {
            profile_scope!("CommandCenter::execute_graphics :: execute");
            let (mut queue, mut pools) = family.acquire(1, device);
            let finish = execution.execute(&mut queue, &mut pools, &*current, &fence, device);
            family.release(
                queue,
                pools,
                fence,
                (finish.0 - current.now().0) as usize,
                device,
            );
            return;
        }
    }

    /// Wait for all commands to finish
    pub fn wait_finish(&mut self, device: &B::Device, current: &mut CurrentEpoch) {
        self.wait_step(device, usize::max_value());
        if let Some(ready) = self.cleanup(device) {
            current.advance(ready as u64);
        }
    }

    /// Check finished operation and advance current epoch.
    fn cleanup(&mut self, device: &B::Device) -> Option<usize> {
        profile_scope!("CommandCenter::cleanup");
        let mut to_clean = None;

        fn get_ready<B: Backend, C>(
            family: &mut Option<Family<B, C>>,
            clean: &mut Option<usize>,
            device: &B::Device,
            fences: &mut Vec<B::Fence>,
        ) {
            if let Some(ref mut family) = family.as_mut() {
                match (family.check_ready(device, fences), *clean) {
                    (Some(checked), Some(ready)) => *clean = Some(min(ready, checked)),
                    (Some(checked), None) => *clean = Some(checked),
                    (None, _) => {}
                }
            }
        }

        get_ready(&mut self.transfer, &mut to_clean, device, &mut self.fences);
        get_ready(&mut self.compute, &mut to_clean, device, &mut self.fences);
        get_ready(&mut self.graphics, &mut to_clean, device, &mut self.fences);
        get_ready(&mut self.general, &mut to_clean, device, &mut self.fences);

        fn shift_ready<B: Backend, C>(family: &mut Option<Family<B, C>>, clean: Option<usize>) {
            if let Some(family) = family.as_mut() {
                family.shift_ready(clean.unwrap_or(usize::max_value()));
            }
        }

        shift_ready(&mut self.transfer, to_clean);
        shift_ready(&mut self.compute, to_clean);
        shift_ready(&mut self.graphics, to_clean);
        shift_ready(&mut self.general, to_clean);

        to_clean
    }

    fn wait_step(&mut self, device: &B::Device, depth: usize) {
        profile_scope!("CommandCenter::wait_step");
        let mut fences: SmallVec<[_; 128]> = SmallVec::new();
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
        if let Some(ready) = self.cleanup(device) {
            current.advance(ready as u64);
        } else if epoch > current.now() {
            let now = current.now();
            current.advance(epoch.0 - now.0);
        }

        while epoch > current.now() {
            self.wait_step(device, min(32, (epoch.0 - current.now().0) as usize));
            if let Some(ready) = self.cleanup(device) {
                current.advance(ready as u64);
            } else {
                let now = current.now();
                current.advance(epoch.0 - now.0);
            }
        }
    }
}
