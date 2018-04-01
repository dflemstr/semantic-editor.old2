//! An executor that is single-threaded, and leverages the browser's `setTimeout` function for
//! scheduling.
use std::cell;
use std::rc;
use std::time;

use futures;
use tokio_executor;

pub mod ffi;
mod current_thread;

/// Runs a future to completion by instantiating a microtask-based executor.  This leverages the
/// browser's event queue to poll futures to completion.
pub fn run<F>(future: F)
where
    F: futures::Future<Item = (), Error = ()> + Send + 'static,
{
    let mut executor = MicrotaskExecutor::new();
    executor.spawn(future);
}

/// Spawns a future on the default executor.  This only works while a default executor is active;
/// for example from within another future.  Behaves mostly like [`tokio::spawn`].
pub fn spawn<F>(future: F)
where
    F: futures::Future<Item = (), Error = ()> + Send + 'static,
{
    tokio_executor::spawn(future)
}

type InnerExecutor = rc::Rc<cell::RefCell<current_thread::CurrentThread<ParkNoop>>>;

/// A unit of work that should be performed by the executor at some point.
#[derive(Clone, Debug)]
pub struct Microtask {
    executor: InnerExecutor,
}

/// An executor for executing work to be run asynchronously.
#[derive(Clone, Debug)]
struct MicrotaskExecutor {
    executor: InnerExecutor,
}

struct ParkNoop;

struct UnparkNoop;

impl MicrotaskExecutor {
    /// Creates a new executor instance.
    pub fn new() -> MicrotaskExecutor {
        let current_thread = current_thread::CurrentThread::new_with_park(ParkNoop);
        let executor = rc::Rc::new(cell::RefCell::new(current_thread));
        MicrotaskExecutor { executor }
    }

    /// Schedules some work to be done in the future.
    fn spawn<F>(&mut self, future: F)
    where
        F: futures::Future<Item = (), Error = ()> + 'static,
    {
        self.executor.borrow_mut().spawn(future);
        let executor = self.executor.clone();
        ffi::scheduleMicrotask(ffi::Microtask(Microtask { executor }));
    }
}

impl tokio_executor::Executor for MicrotaskExecutor {
    /// Schedules some work to be done in the future.
    fn spawn(
        &mut self,
        future: Box<futures::Future<Item = (), Error = ()> + Send>,
    ) -> Result<(), tokio_executor::SpawnError> {
        self.spawn(future);
        Ok(())
    }

    fn status(&self) -> Result<(), tokio_executor::SpawnError> {
        Ok(())
    }
}

impl Microtask {
    /// Runs the microtask.
    pub fn run(&self) -> bool {
        let mut executor = self.executor.borrow_mut();
        let _ = executor.turn(None);
        executor.is_idle()
    }
}

impl tokio_executor::park::Park for ParkNoop {
    type Unpark = UnparkNoop;
    type Error = ();

    fn unpark(&self) -> Self::Unpark {
        UnparkNoop
    }

    fn park(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn park_timeout(&mut self, _duration: time::Duration) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl tokio_executor::park::Unpark for UnparkNoop {
    fn unpark(&self) {
        ()
    }
}
