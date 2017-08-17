use crossbeam::sync::MsQueue;
use futures::{Async, Future, Poll};
use futures::sync::oneshot::{Receiver, channel};
use gfx::traits::Pod;
use renderer::{Error, Material, MaterialBuilder, Mesh, MeshBuilder};

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
pub struct Factory {
    jobs: MsQueue<Box<FnOnce(&mut ::renderer::Factory)>>,
}

impl Factory {
    /// Creates a mesh asynchronously.
    pub fn create_mesh<D, V>(&self, mb: MeshBuilder<D, V>) -> MeshFuture
        where D: AsRef<[V]> + 'static,
              V: ::renderer::VertexFormat + 'static,
    {
        self.execute(move |f| mb.build(f))
    }

    /// Creates a mesh asynchronously.
    pub fn create_material<DA, TA, DE, TE, DN, TN, DM, TM, DR, TR, DO, TO, DC, TC>(
        &self,
        mb: MaterialBuilder<DA, TA, DE, TE, DN, TN, DM, TM, DR, TR, DO, TO, DC, TC>)
        -> MaterialFuture
        where DA: AsRef<[TA]> + 'static,
              TA: Pod + 'static,
              DE: AsRef<[TE]> + 'static,
              TE: Pod + 'static,
              DN: AsRef<[TN]> + 'static,
              TN: Pod + 'static,
              DM: AsRef<[TM]> + 'static,
              TM: Pod + 'static,
              DR: AsRef<[TR]> + 'static,
              TR: Pod + 'static,
              DO: AsRef<[TO]> + 'static,
              TO: Pod + 'static,
              DC: AsRef<[TC]> + 'static,
              TC: Pod + 'static,
    {
        self.execute(|f| mb.build(f))
    }

    /// Execute a closure which takes in the real factory.
    pub fn execute<F, T, E>(&self, fun: F) -> FactoryFuture<T, E>
        where F: FnOnce(&mut ::renderer::Factory) -> Result<T, E> + 'static,
              T: 'static,
              E: 'static,
    {
        let (send, recv) = channel();

        self.jobs.push(Box::new(move |factory| {
            let r = fun(factory);
            let _ = send.send(r);
        }));

        FactoryFuture(recv)
    }
}

/// A material which may not have been created yet.
pub type MaterialFuture = FactoryFuture<Material, Error>;

/// A mesh which may not have been created yet.
pub type MeshFuture = FactoryFuture<Mesh, Error>;
