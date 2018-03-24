//! A scheduler for scheduling futures to be run on shared execution facilities.
use std::cell;
use std::rc;
use std::time;

use futures;
use tokio_executor;

pub mod ffi;
pub mod current_thread;

type InnerExecutor = rc::Rc<cell::RefCell<current_thread::CurrentThread<ParkNoop>>>;

/// A unit of work that should be performed by the executor at some point.
#[derive(Clone, Debug)]
pub struct Microtask {
    executor: InnerExecutor,
}

/// An executor for executing work to be run asynchronously.
#[derive(Clone, Debug)]
pub struct Executor {
    executor: InnerExecutor,
}

struct ParkNoop;
struct UnparkNoop;

impl Executor {
    /// Creates a new executor instance.
    pub fn new() -> Executor {
        let current_thread = current_thread::CurrentThread::new_with_park(ParkNoop);
        let executor = rc::Rc::new(cell::RefCell::new(current_thread));
        Executor { executor }
    }

    /// Schedules some work to be done in the future.
    pub fn spawn<F>(&self, future: F)
        where
            F: futures::Future<Item=(), Error=()> + 'static,
    {
        self.executor.borrow_mut().spawn(future);
        let executor = self.executor.clone();
        debug!("Scheduling microtask for new task");
        ffi::scheduleMicrotask(ffi::Microtask(Microtask { executor }));
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
