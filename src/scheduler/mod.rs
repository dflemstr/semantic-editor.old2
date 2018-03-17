//! A scheduler for scheduling futures to be run on shared execution facilities.
use std::collections;
use std::fmt;
use std::sync;

use futures;

pub mod ffi;

const NEXT_ID: sync::atomic::AtomicUsize = sync::atomic::AtomicUsize::new(0);

struct Task(futures::executor::Spawn<Box<futures::Future<Item = (), Error = ()> + Send>>);

type Handle<A> = sync::Arc<sync::Mutex<A>>;

type TaskHandle = Handle<Task>;
type Tasks = collections::HashMap<usize, TaskHandle>;

/// A unit of work that should be performed by the scheduler at some point.
#[derive(Debug)]
pub struct Microtask {
    tasks: Handle<Tasks>,
    task: TaskHandle,
    notify: futures::executor::NotifyHandle,
    id: usize,
}

/// A scheduler for scheduling work to be run in the background.
#[derive(Clone, Debug)]
pub struct Scheduler(sync::Arc<Inner>);

#[derive(Debug)]
struct Inner {
    tasks: Handle<Tasks>,
    // TODO(dflemstr): this most likely causes a memory leak... Is the drop below enough?
    me: sync::Mutex<Option<futures::executor::NotifyHandle>>,
}

impl fmt::Debug for Task {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Task(_)")
    }
}

impl Scheduler {
    /// Creates a new scheduler instance.
    pub fn new() -> Scheduler {
        let tasks = new_handle(collections::HashMap::new());
        let me = sync::Mutex::new(None);
        let inner = sync::Arc::new(Inner { tasks, me });
        *inner.me.lock().unwrap() = Some(futures::executor::NotifyHandle::from(inner.clone()));
        Scheduler(inner)
    }

    /// Schedules some work to be done in the future.
    pub fn schedule<F>(&self, future: F)
    where
        F: futures::Future<Item = (), Error = ()> + Send + 'static,
    {
        let boxed_future: Box<futures::Future<Item = (), Error = ()> + Send> = Box::new(future);
        let task = futures::executor::spawn(boxed_future);

        let id = NEXT_ID.fetch_add(1, sync::atomic::Ordering::Relaxed);

        {
            let tasks = &mut self.0.tasks.lock().unwrap();
            tasks.insert(id, new_handle(Task(task)));
        }

        futures::executor::Notify::notify(&*self.0, id);
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        match self.0.me.lock() {
            Ok(ref mut me) => **me = None,
            Err(e) => error!("Could not lock scheduler self reference during drop: {}", e),
        }
    }
}

impl futures::executor::Notify for Inner {
    fn notify(&self, id: usize) {
        let tasks = self.tasks.clone();
        let task = tasks.lock().unwrap()[&id].clone();
        let notify = self.me.lock().unwrap().as_ref().unwrap().clone();
        ffi::scheduleMicrotask(ffi::Microtask(Microtask {
            tasks,
            task,
            notify,
            id,
        }));
    }
}

impl Microtask {
    /// Runs the microtask.
    pub fn run(&self) {
        match futures::executor::Spawn::poll_future_notify(
            &mut self.task.lock().unwrap().0,
            &self.notify,
            self.id,
        ) {
            Ok(futures::Async::Ready(())) => {
                self.tasks.lock().unwrap().remove(&self.id);
                debug!("Task {} completed", self.id);
            }
            Ok(futures::Async::NotReady) => {
                debug!("Task {} not ready", self.id);
            }
            Err(()) => {
                self.tasks.lock().unwrap().remove(&self.id);
                error!("Task {} failed", self.id);
            }
        }
    }
}

fn new_handle<A>(value: A) -> Handle<A> {
    sync::Arc::new(sync::Mutex::new(value))
}
