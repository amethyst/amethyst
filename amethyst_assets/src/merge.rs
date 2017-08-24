// FIXME: This is almost a copy for `Merge` from _specs_. Should be moved there

use std::error::Error;
use std::marker::PhantomData;

use futures::{Async, Future};
use specs::{Component, Entity, Entities, Join, RunningTime, System, WriteStorage};


/// A system which merges `Ready` futures into the persistent storage.
/// Please note that your `World` has to contain a component storage
/// for `F` and `F::Item`.
///
/// In case of an error, it will be added to the `Errors` resource.
pub struct Merge<F> {
    future_type: PhantomData<*const F>,
    tmp: Vec<Entity>,
}

unsafe impl<F> Send for Merge<F> {}
unsafe impl<F> Sync for Merge<F> {}

impl<F> Merge<F> {
    /// Creates a new merge system.
    pub fn new() -> Self {
        Merge {
            future_type: PhantomData,
            tmp: Vec::new(),
        }
    }
}

impl<'a, T, E, F> System<'a> for Merge<F>
    where T: Component + Send + Sync + 'static,
          F: Future<Item = T, Error = E> + Component + Send + Sync,
          E: Error,
{
    type SystemData = (Entities<'a>,
                       // FIXME: Here should be some Fetch<'a, Log> to log occured errors
                       WriteStorage<'a, F>,
                       WriteStorage<'a, T>);

    fn run(&mut self, (entities, mut future, mut pers): Self::SystemData) {
        let mut delete = &mut self.tmp;

        for (e, future) in (&*entities, &mut future).join() {
            match future.poll() {
                Ok(Async::Ready(x)) => {
                    pers.insert(e, x);
                    delete.push(e);
                }
                Ok(Async::NotReady) => {}
                Err(err) => {
                    println!("Error occured in asset loading: {}", err);
                    delete.push(e);
                }
            }
        }

        for e in delete.drain(..) {
            future.remove(e);
        }
    }

    fn running_time(&self) -> RunningTime {
        RunningTime::Short
    }
}