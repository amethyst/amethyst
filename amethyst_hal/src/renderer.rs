use std::any::TypeId;
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;

use hal::{Backend, Device as HalDevice};
use hal::command::{Rect, Viewport};
use hal::image::Kind;
use hal::pool::{CommandPool, CommandPoolCreateFlags};
use hal::queue::{General, QueueGroup, RawCommandQueue, RawSubmission};
use hal::window::{Backbuffer, FrameSync, Surface, Swapchain, SwapchainConfig};
use mem::{Factory as FactoryTrait, SmartAllocator, Type as MemType};
use shred::Resources;
use winit::WindowId;
use xfg::{Graph, GraphBuilder, Pass, PassShaders, SuperFrame as XfgFrame};

#[cfg(feature = "gfx-metal")]
use metal;

use Error;
use factory::{Factory, RelevantImage};

struct Gpu<B: Backend> {
    group: QueueGroup<B, General>,
    pools: Vec<CommandPool<B, General>>,
    fences: Vec<B::Fence>,
    semaphores: Vec<B::Semaphore>,
}

struct AutoreleasePool<B> {
    #[cfg(feature = "gfx-metal")]
    autorelease: Option<metal::AutoreleasePool>,
    _pd: PhantomData<*mut B>,
}

#[cfg(feature = "gfx-metal")]
impl<B: 'static> AutoreleasePool<B> {
    #[inline(always)]
    fn new() -> Self {
        AutoreleasePool {
            autorelease: {
                if TypeId::of::<B>() == TypeId::of::<metal::Backend>() {
                    Some(unsafe { metal::AutoreleasePool::new() })
                } else {
                    None
                }
            },
            _pd: PhantomData,
        }
    }

    #[inline(always)]
    fn reset(&mut self) {
        if TypeId::of::<B>() == TypeId::of::<metal::Backend>() {
            unsafe {
                self.autorelease.as_mut().unwrap().reset();
            }
        }
    }
}

#[cfg(not(feature = "gfx-metal"))]
impl<B: 'static> AutoreleasePool<B> {
    #[inline(always)]
    fn new() -> Self {
        AutoreleasePool { _pd: PhantomData }
    }

    #[inline(always)]
    fn reset(&mut self) {}
}

unsafe impl<B> Send for AutoreleasePool<B> {}
unsafe impl<B> Sync for AutoreleasePool<B> {}

#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct RenderId(u64);

pub struct Renderer<B: Backend, P> {
    autorelease: AutoreleasePool<B>,
    queues_usage: Vec<usize>,
    renders: HashMap<RenderId, Render<B, P>>,
    gpu: Gpu<B>,
    counter: u64,
}

impl<B, P> Renderer<B, P>
where
    B: Backend,
{
    /// Creates new render
    pub fn create_render(
        &mut self,
        mut surface: B::Surface,
        config: SwapchainConfig,
        device: &B::Device,
    ) -> RenderId {
        self.counter += 1;
        let id = RenderId(self.counter);
        debug_assert!(self.renders.get(&id).is_none());

        let queue = self.queues_usage
            .iter()
            .enumerate()
            .min_by_key(|&(_, u)| u)
            .map(|(i, _)| i)
            .expect("There are some queues");
        self.queues_usage[queue] += 1;
        let (swapchain, backbuffer) = device.create_swapchain(&mut surface, config);
        let render = Render {
            queue,
            surface,
            swapchain,
            backbuffer,
            active: None,
            graphs: Vec::new(),
            frames: VecDeque::new(),
            jobs: Vec::new(),
        };
        self.renders.insert(id, render);
        id
    }

    /// Remove render
    pub fn remove_render(&mut self, _id: RenderId) {
        unimplemented!()
    }

    /// Add graph to the render
    pub fn add_graph(
        &mut self,
        id: RenderId,
        graph: GraphBuilder<P>,
        device: &B::Device,
        allocator: &mut SmartAllocator<B>,
    ) -> Result<(), Error>
    where
        P: PassShaders<B>,
    {
        let ref mut render = *self.renders
            .get_mut(&id)
            .ok_or(format!("No render with id {:#?}", id))?;
        let graph = graph
            .build(
                device,
                &render.backbuffer,
                |kind, level, format, usage, properties, device| {
                    allocator.create_image(
                        device,
                        (MemType::General, properties),
                        kind,
                        level,
                        format,
                        usage,
                    )
                },
            )
            .map_err(|err| Error::with_chain(err, "Failed to add graph to the render"))?;
        render.graphs.push(graph);
        Ok(())
    }

    /// Create new render system providing it with general queue group and surfaces to draw onto
    pub(crate) fn new(group: QueueGroup<B, General>) -> Self
    where
        P: Send + Sync,
    {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<Self>();

        Renderer {
            autorelease: AutoreleasePool::new(),
            queues_usage: vec![0; group.queues.len()],
            renders: HashMap::new(),
            counter: 0,
            gpu: Gpu {
                group,
                pools: Vec::new(),
                fences: Vec::new(),
                semaphores: Vec::new(),
            },
        }
    }

    pub(crate) fn run<'a>(&mut self, res: &'a Resources, factory: &mut Factory<B>)
    where
        B: Backend,
        P: Pass<B, &'a Resources>,
    {
        self.poll_factory_orders(factory);

        // Run renders
        for r in self.renders.values_mut() {
            r.run(res, factory, &mut self.gpu);
        }

        // walk over frames and find earliest
        let earliest = self.renders
            .values()
            .filter_map(|r| r.frames.front())
            .map(|f| f.started)
            .min()
            .unwrap();

        unsafe {
            // cleanup after finished jobs.
            factory.advance(earliest);
        }

        self.autorelease.reset();
    }

    fn poll_factory_orders(&mut self, factory: &mut Factory<B>)
    where
        B: Backend,
    {
        if let Some((cbuf, _)) = factory.uploads() {
            if self.gpu.group.queues.len() > 1 {
                unimplemented!("Upload in multiqueue environment is not supported yet");
            }
            unsafe {
                self.gpu.group.queues[0].as_mut().submit_raw(
                    RawSubmission {
                        cmd_buffers: Some(cbuf),
                        wait_semaphores: &[],
                        signal_semaphores: &[],
                    },
                    None,
                );
            }
        }
    }
}

struct Render<B: Backend, P> {
    queue: usize,
    surface: B::Surface,
    swapchain: B::Swapchain,
    backbuffer: Backbuffer<B>,
    active: Option<usize>,
    graphs: Vec<Graph<B, RelevantImage<B>, P>>,
    frames: VecDeque<Frame>,
    jobs: Vec<Job<B>>,
}

impl<B, P> Render<B, P>
where
    B: Backend,
{
    fn run<'a>(&mut self, mut res: &'a Resources, factory: &mut Factory<B>, gpu: &mut Gpu<B>)
    where
        P: Pass<B, &'a Resources>,
    {
        if let Some(active) = self.active {
            // Get fresh semaphore.
            let acquire = gpu.semaphores
                .pop()
                .unwrap_or_else(|| factory.create_semaphore());

            // Start frame aquisition.
            let surface_frame = self.swapchain.acquire_frame(FrameSync::Semaphore(&acquire));
            let frame = Frame {
                index: surface_frame.id(),
                started: factory.current(),
            };
            let xfg_frame = XfgFrame::new(&self.backbuffer, surface_frame);

            // Grow job vector.
            while frame.index >= self.jobs.len() {
                self.jobs.push(Job {
                    release: gpu.semaphores
                        .pop()
                        .unwrap_or_else(|| factory.create_semaphore()),
                    payload: None,
                });
            }

            // Pop earliest jobs ...
            while let Some(f) = self.frames.pop_front() {
                // Get the job.
                let ref mut job = self.jobs[f.index];

                if let Some(Payload {
                    fence,
                    mut pool,
                    acquire,
                    ..
                }) = job.payload.take()
                {
                    // Wait for job to finish.
                    if !factory.wait_for_fence(&fence, !0) {
                        panic!("Device lost or something");
                    }
                    // reset fence and pool
                    factory.reset_fence(&fence);
                    pool.reset();

                    // Reclaim fence, pool and acquisition semaphore
                    gpu.fences.push(fence);
                    gpu.pools.push(pool);
                    gpu.semaphores.push(acquire);
                }

                // ... until the job associated with current frame
                if f.index == frame.index {
                    break;
                }
            }

            let ref mut job = self.jobs[frame.index];

            let fence = gpu.fences
                .pop()
                .unwrap_or_else(|| factory.create_fence(false));
            let mut pool = gpu.pools.pop().unwrap_or_else(|| {
                factory.create_command_pool_typed(&gpu.group, CommandPoolCreateFlags::TRANSIENT, 1)
            });

            // Get all required resources.
            let ref mut graph = self.graphs[active];
            let ref mut queue = gpu.group.queues[self.queue];

            // Record and submit commands to draw frame.
            graph.draw_inline(
                queue,
                &mut pool,
                xfg_frame,
                &acquire,
                &job.release,
                viewport(self.surface.kind()),
                &fence,
                &factory,
                &mut res,
            );

            // Setup presenting.
            queue.present(Some(&mut self.swapchain), Some(&job.release));

            // Save job resources.
            job.payload = Some(Payload {
                fence,
                acquire,
                pool,
            });

            // Enque frame.
            self.frames.push_back(frame);
        } else if !self.jobs.is_empty() {
            // Render wants to stop processing.
            // Wait for associated queue to become idle.
            gpu.group.queues[self.queue]
                .wait_idle()
                .expect("Device lost or something");

            // Get all jobs
            for Job { release, payload } in self.jobs.drain(..) {
                if let Some(Payload {
                    fence,
                    mut pool,
                    acquire,
                    ..
                }) = payload
                {
                    // reset fence and pool
                    factory.reset_fence(&fence);
                    pool.reset();

                    // Reclaim fence, pool and semaphores
                    gpu.fences.push(fence);
                    gpu.pools.push(pool);
                    gpu.semaphores.push(acquire);
                    gpu.semaphores.push(release);
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Frame {
    index: usize,
    started: u64,
}

struct Job<B: Backend> {
    release: B::Semaphore,
    payload: Option<Payload<B>>,
}

struct Payload<B: Backend> {
    acquire: B::Semaphore,
    fence: B::Fence,
    pool: CommandPool<B, General>,
}

fn viewport(kind: Kind) -> Viewport {
    match kind {
        Kind::D2(w, h, _) | Kind::D2Array(w, h, _, _) => Viewport {
            rect: Rect { x: 0, y: 0, w, h },
            depth: 0.0..1.0,
        },
        _ => panic!("Unsupported surface kind"),
    }
}
