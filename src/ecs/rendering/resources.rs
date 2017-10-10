//! `amethyst` rendering ecs resources

use std::sync::Arc;

use crossbeam::sync::MsQueue;
use futures::{Async, Future, Poll};
use futures::sync::oneshot::{channel, Receiver, Sender};
use gfx::traits::Pod;
use renderer::{Error, Material, MaterialBuilder, Texture, TextureBuilder};
use renderer::Rgba;
use renderer::mesh::{Mesh, MeshBuilder, VertexDataSet};
use smallvec::SmallVec;
use winit::Window;

pub(crate) trait Exec: Send {
    fn exec(self: Box<Self>, factory: &mut ::renderer::Factory);
}

/// A factory future.
pub struct FactoryFuture<A, E>(Receiver<Result<A, E>>);

impl<A, E> Future for FactoryFuture<A, E> {
    type Item = A;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.0.poll().expect("Sender destroyed") {
            Async::Ready(x) => x.map(Async::Ready),
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

/// The factory abstraction, which allows to access the real
/// factory and returns futures.
#[derive(Clone)]
pub struct Factory {
    pub(crate) jobs: Arc<MsQueue<Box<Exec>>>,
}

impl Factory {
    /// Creates a new factory resource.
    pub fn new() -> Self {
        Factory {
            jobs: Arc::new(MsQueue::new()),
        }
    }

    /// Creates a mesh asynchronously.
    pub fn create_mesh<T>(&self, mb: MeshBuilder<T>) -> MeshFuture
    where
        T: VertexDataSet + Send + 'static,
    {
        self.execute(move |f| mb.build(f))
    }

    /// Creates a texture asynchronously.
    pub fn create_texture<D, T>(&self, tb: TextureBuilder<D, T>) -> TextureFuture
    where
        D: AsRef<[T]> + Send + 'static,
        T: Pod + Send + Copy + 'static,
    {
        self.execute(move |f| tb.build(f))
    }

    /// Creates a mesh asynchronously.
    pub fn create_material<DA, TA, DE, TE, DN, TN, DM, TM, DR, TR, DO, TO, DC, TC>(
        &self,
        mb: MaterialBuilder<DA, TA, DE, TE, DN, TN, DM, TM, DR, TR, DO, TO, DC, TC>,
    ) -> MaterialFuture
    where
        DA: AsRef<[TA]> + Send + 'static,
        TA: Pod + Send + Copy + 'static,
        DE: AsRef<[TE]> + Send + 'static,
        TE: Pod + Send + Copy + 'static,
        DN: AsRef<[TN]> + Send + 'static,
        TN: Pod + Send + Copy + 'static,
        DM: AsRef<[TM]> + Send + 'static,
        TM: Pod + Send + Copy + 'static,
        DR: AsRef<[TR]> + Send + 'static,
        TR: Pod + Send + Copy + 'static,
        DO: AsRef<[TO]> + Send + 'static,
        TO: Pod + Send + Copy + 'static,
        DC: AsRef<[TC]> + Send + 'static,
        TC: Pod + Send + Copy + 'static,
    {
        self.execute(|f| mb.build(f))
    }

    /// Execute a closure which takes in the real factory.
    pub fn execute<F, T, E>(&self, fun: F) -> FactoryFuture<T, E>
    where
        F: FnOnce(&mut ::renderer::Factory) -> Result<T, E> + Send + 'static,
        T: Send + 'static,
        E: Send + 'static,
    {
        let (send, recv) = channel();

        struct Job<F, T, E>(F, Sender<Result<T, E>>);

        impl<F, T, E> Exec for Job<F, T, E>
        where
            F: FnOnce(&mut ::renderer::Factory) -> Result<T, E> + Send + 'static,
            T: Send + 'static,
            E: Send + 'static,
        {
            fn exec(self: Box<Self>, factory: &mut ::renderer::Factory) {
                let job = *self;
                let Job(closure, sender) = job;

                let r = closure(factory);
                let _ = sender.send(r);
            }
        }

        self.jobs.push(Box::new(Job(fun, send)));

        FactoryFuture(recv)
    }
}

/// A texture which may not have been created yet.
pub type TextureFuture = FactoryFuture<Texture, Error>;

/// A material which may not have been created yet.
pub type MaterialFuture = FactoryFuture<Material, Error>;

/// A mesh which may not have been created yet.
pub type MeshFuture = FactoryFuture<Mesh, Error>;

/// The ambient color of a scene
#[derive(Clone, Debug, Default)]
pub struct AmbientColor(pub Rgba);

impl AsRef<Rgba> for AmbientColor {
    fn as_ref(&self) -> &Rgba {
        &self.0
    }
}

/// This specs resource with id 0 permits sending commands to the
/// renderer internal window.
pub struct WindowMessages {
    // It's unlikely we'll get more than one command per frame
    // 1 Box also makes this the same size as a Vec, so this costs
    // no more space in the structure than a Vec would.
    //
    // NOTE TO FUTURE AUTHORS: This could be an FnOnce but that's not possible
    // right now as of 2017-10-02 because FnOnce isn't object safe.  It might
    // be possible as soon as FnBox stabilizes.  For now I'll use FnMut instead.
    pub(crate) queue: SmallVec<[Box<FnMut(&Window) + Send + Sync + 'static>; 2]>,
}

impl WindowMessages {
    /// Create a new `WindowMessages`
    pub fn new() -> Self {
        Self {
            queue: SmallVec::new(),
        }
    }

    /// Execute this closure on the `winit::Window` next frame.
    pub fn send_command<F>(&mut self, command: F)
    where
        F: FnMut(&Window) + Send + Sync + 'static,
    {
        self.queue.push(Box::new(command));
    }
}

/// World resource that stores screen dimensions.
pub struct ScreenDimensions {
    /// Screen width in pixels (px).
    w: f32,
    /// Screen height in pixels (px).
    h: f32,
    /// Width divided by height.
    aspect_ratio: f32,
    pub(crate) dirty: bool,
}

impl ScreenDimensions {
    /// Creates a new screen dimensions object with the given width and height.
    pub fn new(w: u32, h: u32) -> ScreenDimensions {
        ScreenDimensions {
            w: w as f32,
            h: h as f32,
            aspect_ratio: w as f32 / h as f32,
            dirty: false,
        }
    }

    /// Returns the current width of the window.
    pub fn width(&self) -> f32 {
        self.w
    }

    /// Returns the current height of the window.
    pub fn height(&self) -> f32 {
        self.h
    }

    /// Returns the current aspect ratio of the window.
    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    /// Updates the width and height of the screen and recomputes the aspect
    /// ratio.
    pub fn update(&mut self, w: u32, h: u32) {
        self.w = w as f32;
        self.h = h as f32;
        self.aspect_ratio = w as f32 / h as f32;
        self.dirty = true;
    }
}
