use std::collections::{HashMap, VecDeque};

use hal::{Backend, Device as HalDevice};
use hal::command::{Rect, Viewport};
use hal::image::Kind;
use hal::pool::{CommandPool, CommandPoolCreateFlags};
use hal::queue::{General, QueueGroup};
use hal::window::{Backbuffer, FrameSync, Surface, Swapchain};
use mem::{Factory as FactoryTrait, Type as MemType};
use shred::{Resources, RunNow};
use winit::WindowId;
use xfg::SuperFrame as XfgFrame;

#[cfg(feature = "gfx-metal")]
use metal;

use AmethystGraph;
use factory::{BackendEx, Factory};

///
pub struct RenderSystem<B: BackendEx> {
    queues_usage: Vec<usize>,
    group: QueueGroup<B, General>,
    renders: HashMap<WindowId, Render<B>>,
    pools: Vec<CommandPool<B, General>>,
    fences: Vec<B::Fence>,
    semaphores: Vec<B::Semaphore>,

    #[cfg(feature = "gfx-metal")]
    autorelease: Option<metal::AutoreleasePool>,
}


#[cfg(feature = "gfx-metal")]
impl RenderSystem<metal::Backend> {
    fn insert_autorelease(&mut self, autorelease: metal::AutoreleasePool) {
        assert!(self.autorelease.is_none());
        self.autorelease = Some(autorelease);
    }
}

impl<B> RenderSystem<B>
where
    B: BackendEx,
{
    /// Create new render system providing it with general queue group and surfaces to draw onto
    pub(crate) fn new(group: QueueGroup<B, General>) -> Self {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<Render<B>>();

        RenderSystem {
            queues_usage: vec![0; group.queues.len()],
            group,
            renders: HashMap::new(),
            pools: Vec::new(),
            fences: Vec::new(),
            semaphores: Vec::new(),
            #[cfg(feature = "gfx-metal")]
            autorelease: None,
        }
    }

    fn run_renders(&mut self, mut res: &Resources, factory: &mut Factory<B>) {
        let ref mut group = self.group;
        let current = factory.advance();

        // Run renders
        for r in self.renders.values_mut() {
            if let Some(active) = r.active {
                // Get fresh semaphore.
                let acquire = self.semaphores
                    .pop()
                    .unwrap_or_else(|| factory.create_semaphore());

                // Start frame aquisition.
                let surface_frame = r.swapchain.acquire_frame(FrameSync::Semaphore(&acquire));
                let frame = Frame {
                    index: surface_frame.id(),
                    started: current,
                };
                let xfg_frame = XfgFrame::new(&r.backbuffer, surface_frame);

                // Grow job vector.
                while frame.index >= r.jobs.len() {
                    r.jobs.push(Job {
                        release: self.semaphores
                            .pop()
                            .unwrap_or_else(|| factory.create_semaphore()),
                        payload: None,
                    });
                }

                // Pop earliest jobs ...
                while let Some(f) = r.frames.pop_front() {
                    // Get the job.
                    let ref mut job = r.jobs[f.index];

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
                        self.fences.push(fence);
                        self.pools.push(pool);
                        self.semaphores.push(acquire);
                    }

                    // ... until the job associated with current frame
                    if f.index == frame.index {
                        break;
                    }
                }

                let ref mut job = r.jobs[frame.index];

                let fence = self.fences
                    .pop()
                    .unwrap_or_else(|| factory.create_fence(false));
                let mut pool = self.pools.pop().unwrap_or_else(|| {
                    factory.create_command_pool_typed(&group, CommandPoolCreateFlags::TRANSIENT, 1)
                });

                // Get all required resources.
                let ref mut graph = r.graphs[active];
                let ref mut queue = group.queues[r.queue];

                // Record and submit commands to draw frame.
                graph.draw_inline(
                    queue,
                    &mut pool,
                    xfg_frame,
                    &acquire,
                    &job.release,
                    viewport(r.surface.get_kind()),
                    &fence,
                    &factory,
                    &mut res,
                );

                // Setup presenting.
                queue.present(Some(&mut r.swapchain), Some(&job.release));

                // Save job resources.
                job.payload = Some(Payload {
                    fence,
                    acquire,
                    pool,
                });

                // Enque frame.
                r.frames.push_back(frame);
            } else if !r.jobs.is_empty() {
                // Render wants to stop processing.
                // Wait for associated queue to become idle.
                group.queues[r.queue]
                    .wait_idle()
                    .expect("Device lost or something");

                // Get all jobs
                for Job { release, payload } in r.jobs.drain(..) {
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
                        self.fences.push(fence);
                        self.pools.push(pool);
                        self.semaphores.push(acquire);
                        self.semaphores.push(release);
                    }
                }
            }
        }

        // walk over frames and find earliest
        let earliest = self.renders
            .values()
            .filter_map(|r| r.frames.front())
            .map(|f| f.started)
            .min()
            .unwrap();

        // cleanup after finished jobs.
        factory.clean(earliest);
    }

    fn poll_factory_orders(&mut self, factory: &mut Factory<B>) {
        let mut surfaces = Vec::new();
        factory.get_surfaces(&mut surfaces);
        for (window_id, mut surface, config) in surfaces {
            let queue = self.queues_usage
                .iter()
                .enumerate()
                .min_by_key(|&(_, u)| u)
                .map(|(i, _)| i)
                .expect("There are some queues");
            let (swapchain, backbuffer) = factory.create_swapchain(&mut surface, config);
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
            self.renders.insert(window_id, render);
        }

        let mut graphs = Vec::new();
        factory.get_graphs(&mut graphs);
        let (device, allocator) = factory.device_and_allocator();
        for (window_id, graph) in graphs {
            let ref mut render = *match self.renders.get_mut(&window_id) {
                Some(render) => render,
                None => {
                    error!("Failed to add graph. No window: {:#?}", window_id);
                    continue;
                }
            };
            match graph.build(
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
            ) {
                Ok(graph) => {
                    render.graphs.push(graph);
                }
                Err(err) => {
                    error!("Failed to build graph: {:#?}", err);
                }
            };
        }
    }
}

impl<'a, B> RunNow<'a> for RenderSystem<B>
where
    B: BackendEx,
{
    fn run_now(&mut self, res: &'a Resources) {
        let ref mut factory = *res.fetch_mut::<Factory<B>>(0);
        self.poll_factory_orders(factory);
        self.run_renders(res, factory);

        #[cfg(feature = "gfx-metal")]
        {
            if let Some(ref mut autorelease) = self.autorelease {
                unsafe { autorelease.reset() };
            }
        }
    }
}

struct Render<B: BackendEx> {
    queue: usize,
    surface: B::SurfaceEx,
    swapchain: B::SwapchainEx,
    backbuffer: Backbuffer<B>,
    active: Option<usize>,
    graphs: Vec<AmethystGraph<B>>,
    frames: VecDeque<Frame>,
    jobs: Vec<Job<B>>,
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
