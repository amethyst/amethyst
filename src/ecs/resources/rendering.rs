use std::cell::UnsafeCell;
use std::sync::Arc;

use crossbeam::sync::MsQueue;
use futures::{Async, Future, Poll};
use renderer::{Error, Mesh, MeshBuilder};

/// A factory future.
pub struct FactoryFuture<A, E> {
    inner: Option<Arc<FactoryFutureCell<A, E>>>,
}

impl<A: 'static, E: 'static> FactoryFuture<A, E> {
    fn new() -> (Self, Arc<FactoryFutureCell<A, E>>) {
        let inner = FactoryFutureCell { value: UnsafeCell::new(None) };
        let inner = Arc::new(inner);

        let cloned = inner.clone();
        let inner = Some(inner);

        (FactoryFuture { inner }, cloned)
    }
}

impl<A, E> Future for FactoryFuture<A, E> {
    type Item = A;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match Arc::try_unwrap(self.inner.take().unwrap()) {
            Ok(x) => {
                // As soon as the worker thread finished executing the closure, the second
                // reference gets dropped and we enter this branch.
                // Thus, we are the only one with access to the cell.
                unsafe {
                    x.value
                        .into_inner()
                        .expect("Thread panicked")
                        .map(Async::Ready)
                }
            }
            Err(arc) => {
                self.inner = Some(arc);

                Ok(Async::NotReady)
            }
        }
    }

    fn wait(mut self) -> Result<Self::Item, Self::Error> {
        use futures::Async;

        loop {
            match self.poll() {
                Ok(Async::Ready(x)) => return Ok(x),
                Ok(Async::NotReady) => {}
                Err(x) => return Err(x),
            }
        }
    }
}

struct FactoryFutureCell<A, E> {
    value: UnsafeCell<Option<Result<A, E>>>,
}

impl<A, E> FactoryFutureCell<A, E> {
    fn set(&self, res: Result<A, E>) {
        unsafe { *self.value.get() = Some(res); }
    }
}

unsafe impl<A, E> Send for FactoryFutureCell<A, E> {}

unsafe impl<A, E> Sync for FactoryFutureCell<A, E> {}

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

    /// Execute a closure which takes in the real factory.
    pub fn execute<F, T, E>(&self, fun: F) -> FactoryFuture<T, E>
        where F: FnOnce(&mut ::renderer::Factory) -> Result<T, E> + 'static,
              T: 'static,
              E: 'static,
    {
        let (f, cell) = FactoryFuture::new();

        self.jobs.push(Box::new(move |factory| {
            let r = fun(factory);
            cell.set(r);
        }));

        f
    }
}

/// A mesh which may not have been created yet.
pub type MeshFuture = FactoryFuture<Mesh, Error>;
