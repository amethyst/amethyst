//! Copy of `futures::stream::Shared` implementation
//! `UpdatesStream` do the same. And it holds only last value


use std::{fmt, mem, ops};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::atomic::Ordering::SeqCst;

use futures::{Async, Poll, Stream};
use futures::executor::{self, Notify, Spawn};
use futures::task::{self, Task};

use parking_lot::RwLock;

/// A stream that is cloneable and can be polled in multiple threads.
#[must_use = "streams do nothing unless polled"]
pub struct UpdatesStream<S: Stream> {
    inner: Arc<Inner<S>>,
    waiter: usize,
    counter: usize,
}

impl<S> fmt::Debug for UpdatesStream<S>
where
    S: Stream + fmt::Debug,
    S::Item: fmt::Debug,
    S::Error: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("UpdatesStream")
            .field("inner", &self.inner)
            .field("waiter", &self.waiter)
            .finish()
    }
}

struct Inner<S: Stream> {
    next_clone_id: AtomicUsize,
    stream: UnsafeCell<Option<Spawn<S>>>,
    counter: AtomicUsize,
    last: RwLock<Poll<Option<S::Item>, SharedError<S::Error>>>,
    notifier: Arc<Notifier>,
}

struct Notifier {
    state: AtomicUsize,
    waiters: Mutex<HashMap<usize, Task>>,
}

const IDLE: usize = 0;
const POLLING: usize = 1;
const REPOLL: usize = 2;
const COMPLETE: usize = 3;
const POISONED: usize = 4;

impl<S> UpdatesStream<S>
where
    S: Stream,
    S::Item: Clone,
{
    pub fn new(stream: S) -> Self {
        UpdatesStream {
            inner: Arc::new(Inner {
                next_clone_id: AtomicUsize::new(1),
                notifier: Arc::new(Notifier {
                    state: AtomicUsize::new(IDLE),
                    waiters: Mutex::new(HashMap::new()),
                }),
                stream: UnsafeCell::new(Some(executor::spawn(stream))),
                counter: AtomicUsize::new(0),
                last: RwLock::new(Ok(Async::NotReady)),
            }),
            waiter: 0,
            counter: 0,
        }
    }

    /// If any clone of this `UpdatesStream` has completed execution, returns its result immediately
    /// without blocking. Otherwise, returns None without triggering the work represented by
    /// this `UpdatesStream`.
    fn peek(&mut self) -> Poll<Option<S::Item>, SharedError<S::Error>> {
        let last = self.inner.counter.load(Ordering::Acquire);
        if self.counter < last {
            self.counter = last;
            self.inner.last.read().clone()
        } else {
            Ok(Async::NotReady)
        }
    }

    /// Check if this stream is shared
    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.inner) > 1
    }

    fn set_waiter(&mut self) {
        let mut waiters = self.inner.notifier.waiters.lock().unwrap();
        waiters.insert(self.waiter, task::current());
    }

    fn idle(&self) {
        self.inner.notifier.state.store(IDLE, SeqCst);
        self.inner.notifier.notify(0);
    }

    fn complete(&self) {
        unsafe { *self.inner.stream.get() = None; }
        self.inner.notifier.state.store(COMPLETE, SeqCst);
        self.inner.notifier.notify(0);
    }
}

impl<S> Stream for UpdatesStream<S>
where
    S: Stream,
    S::Item: Clone,
    S::Error: Clone,
{
    type Item = S::Item;
    type Error = SharedError<S::Error>;

    fn poll(&mut self) -> Poll<Option<S::Item>, SharedError<S::Error>> {
        self.set_waiter();

        match self.inner
            .notifier
            .state
            .compare_and_swap(IDLE, POLLING, SeqCst)
        {
            IDLE => {
                // Lock acquired, fall through
            }
            POLLING | REPOLL => {
                // Another task is currently polling, at this point we just want
                // to ensure that our task handle is currently registered

                return Ok(Async::NotReady);
            }
            COMPLETE => {
                return Ok(Async::Ready(None));
            }
            POISONED => panic!("inner stream panicked during poll"),
            _ => unreachable!(),
        }

        loop {
            struct Reset<'a>(&'a AtomicUsize);

            impl<'a> Drop for Reset<'a> {
                fn drop(&mut self) {
                    use std::thread;

                    if thread::panicking() {
                        self.0.store(POISONED, SeqCst);
                    }
                }
            }

            let _reset = Reset(&self.inner.notifier.state);

            // Poll the stream
            let res = unsafe {
                (*self.inner.stream.get())
                    .as_mut()
                    .unwrap()
                    .poll_stream_notify(&self.inner.notifier, 0)
            };
            match res {
                Ok(Async::NotReady) => {
                    // Not ready, try to release the handle
                    match self.inner
                        .notifier
                        .state
                        .compare_and_swap(POLLING, IDLE, SeqCst)
                    {
                        POLLING => {
                            // Success
                            return Ok(Async::NotReady);
                        }
                        REPOLL => {
                            // Gotta poll again!
                            let prev = self.inner.notifier.state.swap(POLLING, SeqCst);
                            assert_eq!(prev, REPOLL);
                        }
                        _ => unreachable!(),
                    }
                }
                Ok(Async::Ready(i)) => {
                    let complete = i.is_none();
                    (*self.inner.last.write()) = Ok(Async::Ready(i));
                    self.inner.counter.fetch_add(1, Ordering::Release);
                    if complete {
                        self.complete();
                    } else {
                        self.idle();
                    }
                    break;
                }
                Err(e) => {
                    (*self.inner.last.write()) = Err(SharedError { error: Arc::new(e) });
                    self.inner.counter.fetch_add(1, Ordering::Release);

                    self.complete();
                    break;
                }
            }
        }

        self.peek()
    }
}

impl<S> Clone for UpdatesStream<S>
where
    S: Stream,
{
    fn clone(&self) -> Self {
        let next_clone_id = self.inner.next_clone_id.fetch_add(1, SeqCst);

        UpdatesStream {
            inner: self.inner.clone(),
            waiter: next_clone_id,
            counter: 0,
        }
    }
}

impl<S> Drop for UpdatesStream<S>
where
    S: Stream,
{
    fn drop(&mut self) {
        let mut waiters = self.inner.notifier.waiters.lock().unwrap();
        waiters.remove(&self.waiter);
    }
}

impl Notify for Notifier {
    fn notify(&self, _id: usize) {
        self.state.compare_and_swap(POLLING, REPOLL, SeqCst);

        let waiters = mem::replace(&mut *self.waiters.lock().unwrap(), HashMap::new());

        for (_, waiter) in waiters {
            waiter.notify();
        }
    }
}

unsafe impl<S: Stream> Sync for Inner<S> {}
unsafe impl<S: Stream> Send for Inner<S> {}

impl<S> fmt::Debug for Inner<S>
where
    S: Stream + fmt::Debug,
    S::Item: fmt::Debug,
    S::Error: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Inner").finish()
    }
}


/// A wrapped error of the original future that is clonable and implements Deref
/// for ease of use.
pub struct SharedError<E> {
    error: Arc<E>,
}

impl<E> Clone for SharedError<E> {
    fn clone(&self) -> Self {
        SharedError {
            error: self.error.clone(),
        }
    }
}

impl<E> ops::Deref for SharedError<E> {
    type Target = E;

    fn deref(&self) -> &E {
        &self.error.as_ref()
    }
}

impl<E> fmt::Debug for SharedError<E>
where
    E: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.error.fmt(fmt)
    }
}

impl<E> fmt::Display for SharedError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.error.fmt(fmt)
    }
}

impl<E> Error for SharedError<E>
where
    E: Error,
{
    fn description(&self) -> &str {
        self.error.description()
    }
    fn cause(&self) -> Option<&Error> {
        self.error.cause()
    }
}