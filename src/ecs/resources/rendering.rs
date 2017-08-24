use std::sync::Arc;

use crossbeam::sync::MsQueue;
use futures::{Async, Future, Poll};
use futures::sync::oneshot::{Receiver, Sender, channel};
use gfx::traits::Pod;
use renderer::{Error, Material, MaterialBuilder, Mesh, MeshBuilder, Texture, TextureBuilder};

pub(crate) trait Exec: Send + Sync {
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
    pub fn create_mesh<D, V>(&self, mb: MeshBuilder<D, V>) -> MeshFuture
        where D: AsRef<[V]> + Send + Sync + 'static,
              V: ::renderer::VertexFormat + Send + Sync + 'static,
    {
        self.execute(move |f| mb.build(f))
    }

    /// Creates a texture asynchronously.
    pub fn create_texture<D, T>(&self, tb: TextureBuilder<D, T>) -> TextureFuture
        where D: AsRef<[T]> + Send + Sync + 'static,
              T: Pod + Send + Sync + 'static
    {
        self.execute(move |f| tb.build(f))
    }

    /// Creates a mesh asynchronously.
    pub fn create_material<DA, TA, DE, TE, DN, TN, DM, TM, DR, TR, DO, TO, DC, TC>(
        &self,
        mb: MaterialBuilder<DA, TA, DE, TE, DN, TN, DM, TM, DR, TR, DO, TO, DC, TC>)
        -> MaterialFuture
        where DA: AsRef<[TA]> + Send + Sync + 'static,
              TA: Pod + Send + Sync + 'static,
              DE: AsRef<[TE]> + Send + Sync + 'static,
              TE: Pod + Send + Sync + 'static,
              DN: AsRef<[TN]> + Send + Sync + 'static,
              TN: Pod + Send + Sync + 'static,
              DM: AsRef<[TM]> + Send + Sync + 'static,
              TM: Pod + Send + Sync + 'static,
              DR: AsRef<[TR]> + Send + Sync + 'static,
              TR: Pod + Send + Sync + 'static,
              DO: AsRef<[TO]> + Send + Sync + 'static,
              TO: Pod + Send + Sync + 'static,
              DC: AsRef<[TC]> + Send + Sync + 'static,
              TC: Pod + Send + Sync + 'static,
    {
        self.execute(|f| mb.build(f))
    }

    /// Execute a closure which takes in the real factory.
    pub fn execute<F, T, E>(&self, fun: F) -> FactoryFuture<T, E>
        where F: FnOnce(&mut ::renderer::Factory) -> Result<T, E> + Send + Sync + 'static,
              T: Send + Sync + 'static,
              E: Send + Sync + 'static
    {
        let (send, recv) = channel();

        struct Job<F, T, E>(F, Sender<Result<T, E>>);

        impl<F, T, E> Exec for Job<F, T, E>
            where F: FnOnce(&mut ::renderer::Factory) -> Result<T, E> + Send + Sync + 'static,
                  T: Send + Sync + 'static,
                  E: Send + Sync + 'static,
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
